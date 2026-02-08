#!/bin/sh
# dotsmith installer
# Usage: curl -sSf https://raw.githubusercontent.com/purpleneutral/dotsmith/main/install.sh | sh
#
# Downloads a prebuilt binary if available for your platform.
# Falls back to building from source if Rust is installed.
set -eu

REPO_OWNER="purpleneutral"
REPO_NAME="dotsmith"
REPO_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}"
INSTALL_DIR="${DOTSMITH_INSTALL_DIR:-${HOME}/.local/bin}"
TMPDIR=""

info()  { printf '  \033[36m%-12s\033[0m %s\n' "$1" "$2"; }
warn()  { printf '  \033[33m%-12s\033[0m %s\n' "$1" "$2"; }
err()   { printf '  \033[31merror:\033[0m %s\n' "$1" >&2; exit 1; }

cleanup() {
    [ -n "$TMPDIR" ] && rm -rf "$TMPDIR"
}
trap cleanup EXIT

echo ""
echo "  dotsmith installer"
echo "  ==================="
echo ""

# --- detect platform ---

ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) ARCH="unknown" ;;
esac

info "platform" "${OS}-${ARCH}"

# --- try prebuilt binary first ---

install_prebuilt() {
    ASSET="dotsmith-${OS}-${ARCH}.tar.gz"

    # Get latest release tag
    if ! command -v curl >/dev/null 2>&1; then
        return 1
    fi

    LATEST=$(curl -sSf -o /dev/null -w '%{redirect_url}' \
        "${REPO_URL}/releases/latest" 2>/dev/null | grep -oE '[^/]+$') || return 1

    [ -z "$LATEST" ] && return 1

    DOWNLOAD_URL="${REPO_URL}/releases/download/${LATEST}/${ASSET}"

    info "downloading" "${REPO_NAME} ${LATEST} (${ASSET})"

    TMPDIR=$(mktemp -d)
    if curl -sSfL -o "${TMPDIR}/${ASSET}" "$DOWNLOAD_URL" 2>/dev/null; then
        tar xzf "${TMPDIR}/${ASSET}" -C "$TMPDIR"
        mkdir -p "$INSTALL_DIR"
        cp "${TMPDIR}/dotsmith" "${INSTALL_DIR}/dotsmith"
        chmod 755 "${INSTALL_DIR}/dotsmith"
        return 0
    fi

    return 1
}

# --- build from source ---

install_from_source() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo ""
        echo "  No prebuilt binary for your platform and Rust is not installed."
        echo "  Either:"
        echo "    1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "    2. Download a binary from: ${REPO_URL}/releases"
        echo ""
        exit 1
    fi

    command -v git >/dev/null 2>&1 || err "git is required to build from source"

    info "building" "from source (this may take a few minutes)"

    TMPDIR=$(mktemp -d)
    git clone --depth 1 --quiet "${REPO_URL}.git" "$TMPDIR"
    (cd "$TMPDIR" && cargo build --release --quiet)

    mkdir -p "$INSTALL_DIR"
    cp "${TMPDIR}/target/release/dotsmith" "${INSTALL_DIR}/dotsmith"
    chmod 755 "${INSTALL_DIR}/dotsmith"
}

# --- install ---

if ! install_prebuilt; then
    warn "no binary" "prebuilt binary not available, building from source"
    install_from_source
fi

info "installed" "${INSTALL_DIR}/dotsmith"

# --- verify ---

if "$INSTALL_DIR/dotsmith" --version >/dev/null 2>&1; then
    VERSION=$("$INSTALL_DIR/dotsmith" --version)
    info "verified" "$VERSION"
fi

# Check PATH
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo ""
        warn "PATH" "${INSTALL_DIR} is not in your PATH"
        echo "  add this to your shell profile:"
        echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
esac

echo ""
echo "  get started:"
echo "    dotsmith init"
echo "    dotsmith add tmux"
echo ""
