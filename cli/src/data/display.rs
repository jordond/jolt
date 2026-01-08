use color_eyre::eyre::Result;
use std::process::Command;

/// Scale factor for IOMFBBrightnessLevel (16.16 fixed point: 65536 = 1.0 = 100%)
const BRIGHTNESS_SCALE: f32 = 65536.0;

pub struct DisplayData {
    brightness_percent: f32,
    max_nits: Option<u32>,
}

impl DisplayData {
    pub fn new() -> Result<Self> {
        let mut data = Self {
            brightness_percent: 50.0,
            max_nits: None,
        };

        data.refresh()?;
        Ok(data)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_from_ioreg();
        Ok(())
    }

    fn refresh_from_ioreg(&mut self) {
        if let Ok(output) = Command::new("ioreg")
            .args(["-r", "-c", "IOMobileFramebuffer"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_ioreg_output(&stdout);
        }
    }

    fn parse_ioreg_output(&mut self, output: &str) {
        let mut found_brightness = false;

        for line in output.lines() {
            let line = line.trim();

            // Use first IOMFBBrightnessLevel (internal display)
            if !found_brightness && line.contains("\"IOMFBBrightnessLevel\"") {
                if let Some(val) = extract_number(line) {
                    // 16.16 fixed point to percentage: (val / 65536) * 100
                    let brightness = ((val as f32 / BRIGHTNESS_SCALE) * 100.0).clamp(0.0, 100.0);
                    self.brightness_percent = brightness;
                    found_brightness = true;
                }
            }

            if line.contains("\"limit_max_physical_brightness\"") {
                if let Some(val) = extract_number(line) {
                    let nits = (val as f32 / BRIGHTNESS_SCALE) as u32;
                    if nits > 0 && nits < 10000 {
                        self.max_nits = Some(nits);
                    }
                }
            }
        }
    }

    pub fn brightness_percent(&self) -> f32 {
        self.brightness_percent
    }
}

fn extract_number(line: &str) -> Option<i64> {
    line.split('=').nth(1)?.trim().parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_parsing() {
        let output = r#"
    "IOMFBBrightnessLevel" = 32768
    "limit_max_physical_brightness" = 104857600
        "#;

        let mut data = DisplayData {
            brightness_percent: 0.0,
            max_nits: None,
        };
        data.parse_ioreg_output(output);

        // 32768 / 65536 * 100 = 50%
        assert!(
            (data.brightness_percent - 50.0).abs() < 0.1,
            "Expected ~50%, got {}",
            data.brightness_percent
        );
        // 104857600 / 65536 = 1600 nits
        assert_eq!(data.max_nits, Some(1600));
    }
}
