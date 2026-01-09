#!/usr/bin/env bash
set -e

# Jolt installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/jordond/jolt/main/install.sh | bash
# Usage with prerelease: curl -fsSL https://raw.githubusercontent.com/jordond/jolt/main/install.sh | bash -s -- --prerelease

REPO="jordond/jolt"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="jolt"
PRERELEASE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}==>${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

log_error() {
    echo -e "${RED}Error:${NC} $1" >&2
}

detect_os() {
    local os
    os="$(uname -s)"
    case "${os}" in
        Linux*)     echo "linux";;
        Darwin*)    echo "darwin";;
        *)
            log_error "Unsupported operating system: ${os}"
            exit 1
            ;;
    esac
}

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "${arch}" in
        x86_64)     echo "x86_64";;
        aarch64)    echo "aarch64";;
        arm64)      echo "aarch64";;
        *)
            log_error "Unsupported architecture: ${arch}"
            exit 1
            ;;
    esac
}

detect_linux_variant() {
    if [ "$(detect_os)" != "linux" ]; then
        echo ""
        return
    fi

    if ldd /bin/ls 2>&1 | grep -q 'musl'; then
        echo "musl"
    elif [ -f /lib/ld-musl-x86_64.so.1 ] || [ -f /lib/ld-musl-aarch64.so.1 ]; then
        echo "musl"
    else
        echo "gnu"
    fi
}

get_latest_version_with_assets() {
    local include_prerelease="$1"
    local url="https://api.github.com/repos/${REPO}/releases"

    local response
    if command -v curl >/dev/null 2>&1; then
        response="$(curl -sL "${url}")"
    elif command -v wget >/dev/null 2>&1; then
        response="$(wget -qO- "${url}")"
    else
        log_error "Either curl or wget is required to download jolt"
        exit 1
    fi

    local tags
    tags="$(echo "${response}" | grep '"tag_name":' | sed -E 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/')"
    while IFS= read -r tag; do
        [ -z "${tag}" ] && continue

        local version="${tag#v}"
        local release_section
        release_section="$(echo "${response}" | awk "BEGIN{found=0} /\"tag_name\":[[:space:]]*\"${tag}\"/{found=1} found{print} /^[[:space:]]*\},[[:space:]]*$/{if(found) exit}")"

        local is_prerelease=false
        if echo "${release_section}" | grep -q '"prerelease":[[:space:]]*true'; then
            is_prerelease=true
        fi

        if [ "${include_prerelease}" = "false" ] && [ "${is_prerelease}" = "true" ]; then
            continue
        fi
        # Check if this release has assets (not an empty array)
        if echo "${release_section}" | grep -q '"assets":[[:space:]]*\['; then
            if ! echo "${release_section}" | grep -q '"assets":[[:space:]]*\[\]'; then
                echo "${version}"
                return 0
            fi
        fi
    done <<< "${tags}"

    log_error "No releases with assets found"
    return 1
}

get_latest_version() {
    local include_prerelease="$1"
    get_latest_version_with_assets "${include_prerelease}"
}

build_download_info() {
    local version="$1"
    local os="$2"
    local arch="$3"
    local variant="$4"

    local target
    if [ "${os}" = "darwin" ]; then
        target="apple-darwin"
    else
        if [ "${variant}" = "musl" ]; then
            target="unknown-linux-musl"
        else
            target="unknown-linux-gnu"
        fi
    fi

    local filename="${BINARY_NAME}-${arch}-${target}"
    local url="https://github.com/${REPO}/releases/download/${version}/${filename}"
    local checksum_url="${url}.sha256"

    echo "${url}|${filename}|${checksum_url}"
}

download_file() {
    local url="$1"
    local output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "${url}" -o "${output}"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "${url}" -O "${output}"
    else
        log_error "Either curl or wget is required to download jolt"
        exit 1
    fi
}

verify_checksum() {
    local file="$1"
    local expected_checksum="$2"

    local actual_checksum
    if command -v shasum >/dev/null 2>&1; then
        actual_checksum="$(shasum -a 256 "${file}" | awk '{print $1}')"
    elif command -v sha256sum >/dev/null 2>&1; then
        actual_checksum="$(sha256sum "${file}" | awk '{print $1}')"
    else
        log_warn "Neither shasum nor sha256sum found, skipping checksum verification"
        return 0
    fi

    if [ "${actual_checksum}" = "${expected_checksum}" ]; then
        return 0
    else
        log_error "Checksum verification failed!"
        log_error "Expected: ${expected_checksum}"
        log_error "Got:      ${actual_checksum}"
        return 1
    fi
}

is_daemon_running() {
    if [ -x "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        "${INSTALL_DIR}/${BINARY_NAME}" daemon status >/dev/null 2>&1
        return $?
    fi
    return 1
}

stop_daemon() {
    if is_daemon_running; then
        log_info "Stopping running daemon..."
        if "${INSTALL_DIR}/${BINARY_NAME}" daemon stop >/dev/null 2>&1; then
            log_info "Daemon stopped"
            return 0
        else
            log_warn "Failed to stop daemon gracefully"
            return 1
        fi
    fi
    return 0
}

start_daemon() {
    log_info "Starting daemon..."
    if "${INSTALL_DIR}/${BINARY_NAME}" daemon start >/dev/null 2>&1; then
        log_info "Daemon started"
        return 0
    else
        log_warn "Failed to start daemon"
        return 1
    fi
}

install_jolt() {
    log_info "Installing jolt..."

    # Check if daemon is running and stop it
    local daemon_was_running=false
    if is_daemon_running; then
        daemon_was_running=true
        stop_daemon
    fi

    local os arch variant
    os="$(detect_os)"
    arch="$(detect_arch)"
    variant="$(detect_linux_variant)"

    log_info "Detected platform: ${os}-${arch}${variant:+-${variant}}"

    if [ "${PRERELEASE}" = "true" ]; then
        log_info "Fetching latest version (including prereleases)..."
    else
        log_info "Fetching latest stable version..."
    fi
    local version
    version="$(get_latest_version "${PRERELEASE}")"
    if [ -z "${version}" ]; then
        log_error "Failed to fetch latest version"
        exit 1
    fi
    log_info "Latest version: ${version}"

    local download_info
    download_info="$(build_download_info "${version}" "${os}" "${arch}" "${variant}")"
    IFS='|' read -r url filename checksum_url <<< "${download_info}"

    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "${tmp_dir}"' EXIT

    log_info "Downloading ${filename}..."
    local binary_path="${tmp_dir}/${BINARY_NAME}"
    if ! download_file "${url}" "${binary_path}"; then
        log_error "Failed to download jolt binary"
        exit 1
    fi

    log_info "Verifying checksum..."
    local checksum_path="${tmp_dir}/${filename}.sha256"
    if download_file "${checksum_url}" "${checksum_path}" 2>/dev/null; then
        local expected_checksum
        expected_checksum="$(cat "${checksum_path}" | awk '{print $1}')"
        if ! verify_checksum "${binary_path}" "${expected_checksum}"; then
            exit 1
        fi
        log_info "Checksum verified"
    else
        log_warn "Checksum file not found, skipping verification"
    fi

    log_info "Installing to ${INSTALL_DIR}..."
    mkdir -p "${INSTALL_DIR}"
    chmod +x "${binary_path}"
    mv "${binary_path}" "${INSTALL_DIR}/${BINARY_NAME}"

    log_info "Successfully installed jolt v${version}!"

    # Restart daemon if it was running
    if [ "${daemon_was_running}" = "true" ]; then
        start_daemon
    fi

    if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
        log_warn "${INSTALL_DIR} is not in your PATH"
        log_warn "Add it to your PATH by adding this line to your shell profile:"
        log_warn "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    fi

    log_info "Run 'jolt --help' to get started"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prerelease)
            PRERELEASE=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            log_error "Usage: $0 [--prerelease]"
            exit 1
            ;;
    esac
done

install_jolt
