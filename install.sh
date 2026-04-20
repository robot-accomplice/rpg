#!/bin/sh
# RPG (Rust Password Generator) installer
# Downloads the latest release binary, verifies its SHA256 checksum,
# installs it to ~/.local/bin, and adds that directory to PATH if needed.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/robot-accomplice/rpg/main/install.sh | sh
#
# Options (via environment variables):
#   RPG_VERSION   - specific version to install (e.g., "v2.0.0"); defaults to latest
#   RPG_INSTALL   - install directory; defaults to ~/.local/bin

set -eu

REPO="robot-accomplice/rpg"
INSTALL_DIR="${RPG_INSTALL:-$HOME/.local/bin}"

# --- Helpers ---------------------------------------------------------------

info()  { printf '  \033[1;34m>\033[0m %s\n' "$*"; }
ok()    { printf '  \033[1;32m✓\033[0m %s\n' "$*"; }
err()   { printf '  \033[1;31m✗\033[0m %s\n' "$*" >&2; }
die()   { err "$@"; exit 1; }

need() {
    command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

# --- Detect platform -------------------------------------------------------

detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) die "Unsupported operating system: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        arm64|aarch64) arch="arm64" ;;
        *) die "Unsupported architecture: $arch" ;;
    esac

    # Validate supported combinations
    case "${os}-${arch}" in
        linux-x86_64)   ASSET_SUFFIX="linux-x86_64" ;;
        linux-arm64)    ASSET_SUFFIX="linux-arm64" ;;
        macos-arm64)    ASSET_SUFFIX="macos-arm64" ;;
        macos-x86_64)   ASSET_SUFFIX="macos-x86_64" ;;
        windows-x86_64) ASSET_SUFFIX="windows-x86_64.exe" ;;
        *) die "No prebuilt binary for ${os}-${arch}. Install from source: cargo install rpg-util" ;;
    esac

    info "Detected platform: ${os} ${arch}"
}

# --- Resolve version -------------------------------------------------------

resolve_version() {
    if [ -n "${RPG_VERSION:-}" ]; then
        VERSION="$RPG_VERSION"
    else
        info "Fetching latest release..."
        VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' \
            | head -1 \
            | sed 's/.*"tag_name": *"//;s/".*//')"
        [ -n "$VERSION" ] || die "Could not determine latest version"
    fi
    ok "Version: $VERSION"
    ASSET="rpg-${VERSION}-${ASSET_SUFFIX}"
}

# --- Download & verify -----------------------------------------------------

download_and_verify() {
    local base_url tmpdir binary_url checksum_url expected_hash actual_hash

    base_url="https://github.com/${REPO}/releases/download/${VERSION}"
    binary_url="${base_url}/${ASSET}"
    checksum_url="${base_url}/${ASSET}.sha256"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    info "Downloading ${ASSET}..."
    curl -fSL --progress-bar -o "${tmpdir}/${ASSET}" "$binary_url" \
        || die "Download failed. Does release $VERSION exist with asset $ASSET?"

    info "Downloading checksum..."
    curl -fsSL -o "${tmpdir}/${ASSET}.sha256" "$checksum_url" \
        || die "Checksum download failed"

    info "Verifying SHA256 checksum..."
    expected_hash="$(awk '{print $1}' "${tmpdir}/${ASSET}.sha256")"

    if command -v sha256sum >/dev/null 2>&1; then
        actual_hash="$(sha256sum "${tmpdir}/${ASSET}" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        actual_hash="$(shasum -a 256 "${tmpdir}/${ASSET}" | awk '{print $1}')"
    else
        die "No SHA256 tool found (need sha256sum or shasum)"
    fi

    if [ "$expected_hash" != "$actual_hash" ]; then
        err "Checksum mismatch!"
        err "  Expected: $expected_hash"
        err "  Actual:   $actual_hash"
        die "Binary may be corrupted or tampered with. Aborting."
    fi
    ok "Checksum verified: ${expected_hash}"

    # Move binary into place
    mkdir -p "$INSTALL_DIR"
    mv "${tmpdir}/${ASSET}" "${INSTALL_DIR}/rpg"
    chmod +x "${INSTALL_DIR}/rpg"
    ok "Installed to ${INSTALL_DIR}/rpg"
}

# --- PATH setup ------------------------------------------------------------

ensure_path() {
    # Already on PATH? Nothing to do.
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) return ;;
    esac

    info "${INSTALL_DIR} is not in PATH. Adding it..."

    local shell_name rc_file export_line
    shell_name="$(basename "${SHELL:-/bin/sh}")"
    export_line="export PATH=\"${INSTALL_DIR}:\$PATH\""

    case "$shell_name" in
        zsh)
            rc_file="$HOME/.zshrc"
            ;;
        bash)
            # Prefer .bashrc; fall back to .bash_profile on macOS
            if [ -f "$HOME/.bashrc" ]; then
                rc_file="$HOME/.bashrc"
            else
                rc_file="$HOME/.bash_profile"
            fi
            ;;
        fish)
            rc_file="$HOME/.config/fish/config.fish"
            export_line="fish_add_path ${INSTALL_DIR}"
            ;;
        *)
            rc_file="$HOME/.profile"
            ;;
    esac

    # Don't duplicate if already present
    if [ -f "$rc_file" ] && grep -qF "$INSTALL_DIR" "$rc_file" 2>/dev/null; then
        ok "PATH entry already in ${rc_file}"
        return
    fi

    printf '\n# Added by rpg installer\n%s\n' "$export_line" >> "$rc_file"
    ok "Added to ${rc_file}"
    info "Restart your shell or run: $export_line"
}

# --- Main ------------------------------------------------------------------

main() {
    printf '\n\033[1m  RPG Installer\033[0m\n\n'

    need curl

    detect_platform
    resolve_version
    download_and_verify
    ensure_path

    printf '\n  \033[1;32mDone!\033[0m Run \033[1mrpg --version\033[0m to verify.\n\n'
}

main "$@"
