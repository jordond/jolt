use std::collections::{HashMap, VecDeque};
use std::ffi::c_void;
use std::mem::size_of;
use std::process::Command;
use std::ptr::null;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use core_foundation_sys::base::{kCFAllocatorDefault, kCFAllocatorNull, CFRelease, CFTypeRef};
use core_foundation_sys::dictionary::{
    CFDictionaryCreateMutableCopy, CFDictionaryGetCount, CFDictionaryGetValue, CFDictionaryRef,
    CFMutableDictionaryRef,
};
use core_foundation_sys::string::{
    kCFStringEncodingUTF8, CFStringCreateWithBytesNoCopy, CFStringGetCString, CFStringRef,
};

use crate::power::{PowerInfo, PowerProvider};
use crate::types::PowerMode;

const SMOOTHING_SAMPLE_COUNT: usize = 5;
const MIN_WARMUP_SAMPLES: usize = 3;

#[derive(Debug, Clone, Copy)]
struct PowerSample {
    cpu_power: f32,
    gpu_power: f32,
    system_power: f32,
}

type IOReportSubscriptionRef = *const c_void;
type CFArrayRef = *const c_void;

#[link(name = "IOReport", kind = "dylib")]
extern "C" {
    fn IOReportCopyChannelsInGroup(
        a: CFStringRef,
        b: CFStringRef,
        c: u64,
        d: u64,
        e: u64,
    ) -> CFDictionaryRef;

    fn IOReportCreateSubscription(
        a: *const c_void,
        b: CFMutableDictionaryRef,
        c: *mut CFMutableDictionaryRef,
        d: u64,
        e: *const c_void,
    ) -> IOReportSubscriptionRef;

    fn IOReportCreateSamples(
        a: IOReportSubscriptionRef,
        b: CFMutableDictionaryRef,
        c: *const c_void,
    ) -> CFDictionaryRef;

    fn IOReportCreateSamplesDelta(
        a: CFDictionaryRef,
        b: CFDictionaryRef,
        c: *const c_void,
    ) -> CFDictionaryRef;

    fn IOReportChannelGetGroup(a: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetChannelName(a: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetUnitLabel(a: CFDictionaryRef) -> CFStringRef;
    fn IOReportSimpleGetIntegerValue(a: CFDictionaryRef, b: i32) -> i64;
}

extern "C" {
    fn CFArrayGetCount(arr: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const c_void;
}

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceMatching(name: *const i8) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        mainPort: u32,
        matching: CFDictionaryRef,
        existing: *mut u32,
    ) -> i32;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IORegistryEntryGetName(entry: u32, name: *mut i8) -> i32;
    fn IOServiceOpen(device: u32, a: u32, b: u32, c: *mut u32) -> i32;
    fn IOServiceClose(conn: u32) -> i32;
    fn IOObjectRelease(obj: u32) -> u32;
    fn IOConnectCallStructMethod(
        conn: u32,
        selector: u32,
        ival: *const c_void,
        isize: usize,
        oval: *mut c_void,
        osize: *mut usize,
    ) -> i32;
    fn mach_task_self() -> u32;
}

#[repr(C)]
#[derive(Debug, Default)]
struct KeyDataVer {
    major: u8,
    minor: u8,
    build: u8,
    reserved: u8,
    release: u16,
}

#[repr(C)]
#[derive(Debug, Default)]
struct PLimitData {
    version: u16,
    length: u16,
    cpu_p_limit: u32,
    gpu_p_limit: u32,
    mem_p_limit: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct KeyInfo {
    data_size: u32,
    data_type: u32,
    data_attributes: u8,
}

#[repr(C)]
#[derive(Debug, Default)]
struct KeyData {
    key: u32,
    vers: KeyDataVer,
    p_limit_data: PLimitData,
    key_info: KeyInfo,
    result: u8,
    status: u8,
    data8: u8,
    data32: u32,
    bytes: [u8; 32],
}

struct Smc {
    conn: u32,
    keys: HashMap<u32, KeyInfo>,
}

impl Smc {
    fn new() -> Option<Self> {
        let service_name = std::ffi::CString::new("AppleSMC").ok()?;

        unsafe {
            let service = IOServiceMatching(service_name.as_ptr());
            let mut existing = 0u32;
            if IOServiceGetMatchingServices(0, service, &mut existing) != 0 {
                return None;
            }

            let mut conn = 0u32;
            loop {
                let device = IOIteratorNext(existing);
                if device == 0 {
                    break;
                }

                let mut name = [0i8; 128];
                if IORegistryEntryGetName(device, name.as_mut_ptr()) == 0 {
                    let name_str = std::ffi::CStr::from_ptr(name.as_ptr())
                        .to_string_lossy()
                        .to_string();
                    if name_str == "AppleSMCKeysEndpoint"
                        && IOServiceOpen(device, mach_task_self(), 0, &mut conn) == 0
                    {
                        IOObjectRelease(existing);
                        return Some(Self {
                            conn,
                            keys: HashMap::new(),
                        });
                    }
                }
            }

            IOObjectRelease(existing);
            None
        }
    }

    fn read(&self, input: &KeyData) -> Option<KeyData> {
        let ival = input as *const _ as _;
        let ilen = size_of::<KeyData>();
        let mut oval = KeyData::default();
        let mut olen = size_of::<KeyData>();

        let rs = unsafe {
            IOConnectCallStructMethod(
                self.conn,
                2,
                ival,
                ilen,
                &mut oval as *mut _ as _,
                &mut olen,
            )
        };

        if rs != 0 || oval.result != 0 {
            return None;
        }

        Some(oval)
    }

    fn read_key_info(&mut self, key: &str) -> Option<KeyInfo> {
        if key.len() != 4 {
            return None;
        }

        let key_fourcc = key.bytes().fold(0u32, |acc, x| (acc << 8) + x as u32);
        if let Some(ki) = self.keys.get(&key_fourcc) {
            return Some(*ki);
        }

        let ival = KeyData {
            data8: 9,
            key: key_fourcc,
            ..Default::default()
        };
        let oval = self.read(&ival)?;
        self.keys.insert(key_fourcc, oval.key_info);
        Some(oval.key_info)
    }

    fn read_val(&mut self, key: &str) -> Option<Vec<u8>> {
        let key_info = self.read_key_info(key)?;
        let key_fourcc = key.bytes().fold(0u32, |acc, x| (acc << 8) + x as u32);

        let ival = KeyData {
            data8: 5,
            key: key_fourcc,
            key_info,
            ..Default::default()
        };
        let oval = self.read(&ival)?;

        Some(oval.bytes[0..key_info.data_size as usize].to_vec())
    }

    fn read_system_power(&mut self) -> Option<f32> {
        let data = self.read_val("PSTR")?;
        if data.len() >= 4 {
            Some(f32::from_le_bytes([data[0], data[1], data[2], data[3]]))
        } else {
            None
        }
    }
}

impl Drop for Smc {
    fn drop(&mut self) {
        unsafe {
            IOServiceClose(self.conn);
        }
    }
}

fn cfstr(val: &str) -> CFStringRef {
    unsafe {
        CFStringCreateWithBytesNoCopy(
            kCFAllocatorDefault,
            val.as_ptr(),
            val.len() as isize,
            kCFStringEncodingUTF8,
            0,
            kCFAllocatorNull,
        )
    }
}

fn from_cfstr(val: CFStringRef) -> String {
    if val.is_null() {
        return String::new();
    }
    unsafe {
        let mut buf = [0i8; 128];
        if CFStringGetCString(val, buf.as_mut_ptr(), 128, kCFStringEncodingUTF8) == 0 {
            return String::new();
        }
        std::ffi::CStr::from_ptr(buf.as_ptr())
            .to_string_lossy()
            .to_string()
    }
}

fn cfdict_get_val(dict: CFDictionaryRef, key: &str) -> Option<CFTypeRef> {
    unsafe {
        let key = cfstr(key);
        let val = CFDictionaryGetValue(dict, key as _);
        CFRelease(key as _);
        if val.is_null() {
            None
        } else {
            Some(val)
        }
    }
}

struct IOReportIterator {
    sample: CFDictionaryRef,
    items: CFArrayRef,
    index: isize,
    count: isize,
}

impl IOReportIterator {
    fn new(sample: CFDictionaryRef) -> Option<Self> {
        let items = cfdict_get_val(sample, "IOReportChannels")? as CFArrayRef;
        let count = unsafe { CFArrayGetCount(items) };
        Some(Self {
            sample,
            items,
            index: 0,
            count,
        })
    }
}

impl Drop for IOReportIterator {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.sample as _);
        }
    }
}

struct ChannelData {
    group: String,
    channel: String,
    unit: String,
    value: i64,
}

impl Iterator for IOReportIterator {
    type Item = ChannelData;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let item = unsafe { CFArrayGetValueAtIndex(self.items, self.index) } as CFDictionaryRef;
        self.index += 1;

        if item.is_null() {
            return self.next();
        }

        let group = from_cfstr(unsafe { IOReportChannelGetGroup(item) });
        let channel = from_cfstr(unsafe { IOReportChannelGetChannelName(item) });
        let unit = from_cfstr(unsafe { IOReportChannelGetUnitLabel(item) })
            .trim()
            .to_string();
        let value = unsafe { IOReportSimpleGetIntegerValue(item, 0) };

        Some(ChannelData {
            group,
            channel,
            unit,
            value,
        })
    }
}

struct IOReportSubscription {
    subscription: IOReportSubscriptionRef,
    channels: CFMutableDictionaryRef,
}

impl IOReportSubscription {
    fn new() -> Option<Self> {
        unsafe {
            let group = cfstr("Energy Model");
            let chan = IOReportCopyChannelsInGroup(group, null(), 0, 0, 0);
            CFRelease(group as _);

            if chan.is_null() {
                return None;
            }

            if cfdict_get_val(chan, "IOReportChannels").is_none() {
                CFRelease(chan as _);
                return None;
            }

            let count = CFDictionaryGetCount(chan);
            let channels = CFDictionaryCreateMutableCopy(kCFAllocatorDefault, count, chan);
            CFRelease(chan as _);

            if channels.is_null() {
                return None;
            }

            let mut sub_dict: CFMutableDictionaryRef = null::<c_void>() as _;
            let subscription =
                IOReportCreateSubscription(null(), channels, &mut sub_dict, 0, null());

            if subscription.is_null() {
                CFRelease(channels as _);
                return None;
            }

            Some(Self {
                subscription,
                channels,
            })
        }
    }

    fn sample(&self) -> Option<CFDictionaryRef> {
        let sample = unsafe { IOReportCreateSamples(self.subscription, self.channels, null()) };
        if sample.is_null() {
            None
        } else {
            Some(sample)
        }
    }
}

impl Drop for IOReportSubscription {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.channels as _);
        }
    }
}

pub struct MacOSPower {
    info: PowerInfo,
    subscription: Option<IOReportSubscription>,
    smc: Option<Smc>,
    last_sample: Option<CFDictionaryRef>,
    last_sample_time: Option<Instant>,
    cpu_power: f32,
    gpu_power: f32,
    ane_power: f32,
    package_power: f32,
    system_power: f32,
    samples: VecDeque<PowerSample>,
}

impl PowerProvider for MacOSPower {
    fn new() -> Result<Self> {
        let subscription = IOReportSubscription::new();
        let smc = Smc::new();

        let mut provider = Self {
            info: PowerInfo::default(),
            subscription,
            smc,
            last_sample: None,
            last_sample_time: None,
            cpu_power: 0.0,
            gpu_power: 0.0,
            ane_power: 0.0,
            package_power: 0.0,
            system_power: 0.0,
            samples: VecDeque::with_capacity(SMOOTHING_SAMPLE_COUNT),
        };

        if let Some(ref sub) = provider.subscription {
            if let Some(sample1) = sub.sample() {
                std::thread::sleep(Duration::from_millis(100));
                if let Some(sample2) = sub.sample() {
                    let elapsed = Duration::from_millis(100);
                    provider.calculate_power_from_delta(sample1, sample2, elapsed);
                    provider.last_sample = Some(sample2);
                    provider.last_sample_time = Some(Instant::now());
                } else {
                    unsafe { CFRelease(sample1 as _) };
                }
            }
        }

        provider.refresh_system_power();
        provider.refresh_power_mode();
        provider.record_sample();
        provider.update_info();
        Ok(provider)
    }

    fn refresh(&mut self) -> Result<()> {
        self.refresh_power_metrics();
        self.refresh_system_power();
        self.refresh_power_mode();
        self.record_sample();
        self.update_info();
        Ok(())
    }

    fn info(&self) -> &PowerInfo {
        &self.info
    }
}

impl MacOSPower {
    fn update_info(&mut self) {
        self.info.cpu_power_watts = self.smoothed_value(|s| s.cpu_power);
        self.info.gpu_power_watts = self.smoothed_value(|s| s.gpu_power);
        self.info.system_power_watts = self.smoothed_value(|s| s.system_power);
        self.info.is_warmed_up = self.samples.len() >= MIN_WARMUP_SAMPLES;
    }

    fn record_sample(&mut self) {
        let sample = PowerSample {
            cpu_power: self.cpu_power,
            gpu_power: self.gpu_power,
            system_power: self.system_power,
        };

        if self.samples.len() >= SMOOTHING_SAMPLE_COUNT {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    fn smoothed_value<F>(&self, extractor: F) -> f32
    where
        F: Fn(&PowerSample) -> f32,
    {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(extractor).sum();
        sum / self.samples.len() as f32
    }

    fn refresh_system_power(&mut self) {
        if let Some(ref mut smc) = self.smc {
            if let Some(power) = smc.read_system_power() {
                self.system_power = power.max(self.package_power);
                return;
            }
        }
        self.system_power = self.package_power;
    }

    fn refresh_power_metrics(&mut self) {
        let Some(ref subscription) = self.subscription else {
            self.fallback_power_estimate();
            return;
        };

        let Some(current_sample) = subscription.sample() else {
            self.fallback_power_estimate();
            return;
        };

        let (Some(prev_sample), Some(prev_time)) = (self.last_sample, self.last_sample_time) else {
            self.last_sample = Some(current_sample);
            self.last_sample_time = Some(Instant::now());
            return;
        };

        let elapsed = prev_time.elapsed();
        self.calculate_power_from_delta(prev_sample, current_sample, elapsed);

        unsafe {
            CFRelease(prev_sample as _);
        }

        self.last_sample = Some(current_sample);
        self.last_sample_time = Some(Instant::now());
    }

    fn calculate_power_from_delta(
        &mut self,
        prev: CFDictionaryRef,
        current: CFDictionaryRef,
        elapsed: Duration,
    ) {
        let delta = unsafe { IOReportCreateSamplesDelta(prev, current, null()) };
        if delta.is_null() {
            self.fallback_power_estimate();
            return;
        }

        let elapsed_ms = elapsed.as_millis() as u64;
        if elapsed_ms == 0 {
            unsafe { CFRelease(delta as _) };
            return;
        }

        let mut cpu_power: f32 = 0.0;
        let mut gpu_power: f32 = 0.0;
        let mut ane_power: f32 = 0.0;
        let mut other_power: f32 = 0.0;

        if let Some(iter) = IOReportIterator::new(delta) {
            for ch in iter {
                if ch.group != "Energy Model" {
                    continue;
                }

                let watts = match energy_to_watts(ch.value, &ch.unit, elapsed_ms) {
                    Some(w) => w,
                    None => continue,
                };

                let channel_lower = ch.channel.to_lowercase();
                if channel_lower.contains("gpu") {
                    gpu_power += watts;
                } else if channel_lower.contains("cpu") || channel_lower.starts_with("pacc") {
                    cpu_power += watts;
                } else if channel_lower.starts_with("ane") {
                    ane_power += watts;
                } else if channel_lower.contains("amcc")
                    || channel_lower.contains("dcs")
                    || channel_lower.contains("dram")
                    || channel_lower.contains("isp")
                    || channel_lower.contains("pmp")
                    || channel_lower.contains("nub")
                    || channel_lower.contains("soc")
                {
                    other_power += watts;
                }
            }
        }

        self.cpu_power = cpu_power;
        self.gpu_power = gpu_power;
        self.ane_power = ane_power;
        self.package_power = cpu_power + gpu_power + ane_power + other_power;
    }

    fn fallback_power_estimate(&mut self) {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(Duration::from_millis(50));
        sys.refresh_all();

        let cpu_usage: f32 =
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

        let base_power = 2.0;
        let max_cpu_power = 15.0;
        self.cpu_power = base_power + (cpu_usage / 100.0) * max_cpu_power;
        self.gpu_power = 1.0;
        self.ane_power = 0.0;
        self.package_power = self.cpu_power + self.gpu_power;
    }

    fn refresh_power_mode(&mut self) {
        if let Ok(output) = Command::new("pmset").args(["-g"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            if stdout.contains("lowpowermode 1") {
                self.info.power_mode = PowerMode::LowPower;
            } else if stdout.contains("highpowermode 1") {
                self.info.power_mode = PowerMode::HighPerformance;
            } else {
                self.info.power_mode = PowerMode::Automatic;
            }
        }
    }
}

impl Drop for MacOSPower {
    fn drop(&mut self) {
        if let Some(sample) = self.last_sample {
            unsafe {
                CFRelease(sample as _);
            }
        }
    }
}

fn energy_to_watts(value: i64, unit: &str, duration_ms: u64) -> Option<f32> {
    let val = value as f32;
    let duration_sec = duration_ms as f32 / 1000.0;

    let watts = match unit {
        "mJ" => val / 1_000.0 / duration_sec,
        "uJ" => val / 1_000_000.0 / duration_sec,
        "nJ" => val / 1_000_000_000.0 / duration_sec,
        _ => return None,
    };

    Some(watts)
}
