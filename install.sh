#!/usr/bin/env bash
#
# recall - one-line installer
#
#   curl -fsSL https://GVenkatesh-12.github.io/recall/install.sh | bash
#
# Downloads the latest (or pinned) recall release binary for your OS/arch,
# verifies its SHA-256 checksum, and installs it locally.
#
# Customize with environment variables:
#   RECALL_VERSION       Pin a version tag, e.g. RECALL_VERSION=v1.0.0 (default: latest)
#   RECALL_INSTALL_DIR   Install directory (default: $HOME/.local/bin)
#   RECALL_OWNER         GitHub owner (default: GVenkatesh-12)
#   RECALL_REPO          GitHub repository (default: recall)
#
set -euo pipefail

RECALL_OWNER="${RECALL_OWNER:-GVenkatesh-12}"
RECALL_REPO="${RECALL_REPO:-recall}"
RECALL_VERSION="${RECALL_VERSION:-latest}"
BASE_URL="https://github.com/${RECALL_OWNER}/${RECALL_REPO}"

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------
say() { printf "\033[1;32mrecall\033[0m %s\n" "$*"; }
warn() { printf "\033[1;33mrecall:\033[0m %s\n" "$*" >&2; }
die() { printf "\033[1;31mrecall:\033[0m %s\n" "$*" >&2; exit 1; }

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

# ---------------------------------------------------------------------------
# platform detection
# ---------------------------------------------------------------------------
detect_os() {
    local uname_os
    uname_os="$(uname -s)"
    case "$uname_os" in
        Linux*)  echo linux ;;
        Darwin*) echo darwin ;;
        MINGW*|MSYS*|CYGWIN*) echo windows ;;
        *) die "unsupported operating system: $uname_os" ;;
    esac
}

detect_arch() {
    local uname_m
    uname_m="$(uname -m)"
    case "$uname_m" in
        x86_64|amd64)    echo x86_64 ;;
        arm64|aarch64)   echo aarch64 ;;
        *) die "unsupported architecture: $uname_m" ;;
    esac
}

# ---------------------------------------------------------------------------
# resolve the version to install
# ---------------------------------------------------------------------------
resolve_version() {
    if [ "$RECALL_VERSION" = "latest" ]; then
        need_cmd curl
        local tag
        tag="$(curl -fsSL -H "User-Agent: recall-installer" "https://api.github.com/repos/${RECALL_OWNER}/${RECALL_REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)"
        [ -n "$tag" ] || die "could not determine the latest release tag for ${RECALL_OWNER}/${RECALL_REPO}"
        echo "$tag"
    else
        echo "$RECALL_VERSION"
    fi
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
main() {
    # accept install directory override via --dir (flag or env var)
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --dir) RECALL_INSTALL_DIR="$2"; shift 2 ;;
            --dir=*) RECALL_INSTALL_DIR="${1#--dir=}"; shift ;;
            *) shift ;;
        esac
    done

    RECALL_INSTALL_DIR="${RECALL_INSTALL_DIR:-$HOME/.local/bin}"

    need_cmd curl
    need_cmd uname

    os="$(detect_os)"
    arch="$(detect_arch)"
    version="$(resolve_version)"

    case "$os:$arch" in
        linux:x86_64)   target="x86_64-unknown-linux-gnu" ;;
        linux:aarch64)  target="aarch64-unknown-linux-gnu" ;;
        darwin:x86_64)  target="x86_64-apple-darwin" ;;
        darwin:aarch64) target="aarch64-apple-darwin" ;;
        windows:x86_64) target="x86_64-pc-windows-msvc" ;;
        *) die "no prebuilt recall binary for ${os}/${arch}" ;;
    esac

    asset_name="recall-${target}.tar.gz"
    [ "$os" = "windows" ] && asset_name="recall-${target}.zip"

    asset_url="${BASE_URL}/releases/download/${version}/${asset_name}"
    sha_url="${asset_url}.sha256"
    install_dir="${RECALL_INSTALL_DIR}"

    say "Installing recall ${version} (${target}) to ${install_dir}"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    say "Downloading ${asset_name} ..."
    curl -fsSL --retry 3 -o "${tmpdir}/${asset_name}" "$asset_url" \
        || die "download failed: $asset_url"
    curl -fsSL --retry 3 -o "${tmpdir}/${asset_name}.sha256" "$sha_url" \
        || die "checksum download failed: $sha_url"

    say "Verifying SHA-256 checksum ..."
    (cd "$tmpdir"
     # normalise the checksum file: strip any leading directory paths
     # so it always references the bare asset filename
     if command -v sed >/dev/null 2>&1; then
         sed -i.bak -E 's#^([0-9a-fA-F]{64})[[:space:]]+(.*/)?([^[:space:]]+)$#\1  \3#' "${asset_name}.sha256" 2>/dev/null \
             && rm -f "${asset_name}.sha256.bak" || true
     fi
     if command -v sha256sum >/dev/null 2>&1; then
         sha256sum -c "${asset_name}.sha256"
     elif command -v shasum >/dev/null 2>&1; then
         shasum -a 256 -c "${asset_name}.sha256"
     else
         die "no sha256 checksum tool found"
     fi) || die "checksum verification failed"

    say "Extracting ..."
    if [ "$os" = "windows" ]; then
        need_cmd unzip
        (cd "$tmpdir" && unzip -o "${asset_name}" >/dev/null)
        exe="$(find "$tmpdir" -maxdepth 2 -type f -name 'recall.exe' | head -n1)"
    else
        (cd "$tmpdir" && tar -xzf "${asset_name}")
        exe="$(find "$tmpdir" -maxdepth 2 -type f -name 'recall' | head -n1)"
    fi
    [ -n "$exe" ] || die "archive did not contain the recall binary"

    mkdir -p "$install_dir"

    if [ -w "$install_dir" ]; then
        install -m 0755 "$exe" "${install_dir}/recall"
    else
        warn "${install_dir} is not writable by the current user; retrying with sudo"
        need_cmd sudo
        sudo install -m 0755 -o "$(id -un)" -g "$(id -gn)" "$exe" "${install_dir}/recall" \
            || sudo install -m 0755 "$exe" "${install_dir}/recall"
    fi

    say "Installed recall to ${install_dir}/recall"

    case ":$PATH:" in
        *":${install_dir}:"*) ;;
        *) warn "${install_dir} is not on your PATH." ;;
    esac

    if [ -x "${install_dir}/recall" ]; then
        say "Done! Run '${install_dir}/recall --version' to verify."
    fi
}

main "$@"