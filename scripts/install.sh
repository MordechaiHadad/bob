#!/usr/bin/env bash
set -e

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  PLATFORM="linux" ;;
    Darwin) PLATFORM="macos" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm" ;;
    *) echo "Unsupported Architecture: $ARCH"; exit 1 ;;
esac

# Construct the asset name pattern to look for
# Example: bob-linux-x86_64.zip or bob-macos-arm.zip
ASSET_PATTERN="bob-${PLATFORM}-${ARCH}.zip"

echo "Fetching latest release for $PLATFORM-$ARCH..."

# Get the download URL from the GitHub API
# We use grep/sed here to avoid needing 'jq' installed
DOWNLOAD_URL=$(curl -s https://api.github.com/repos/MordechaiHadad/bob/releases/latest | \
    grep "browser_download_url" | \
    grep "$ASSET_PATTERN" | \
    head -n 1 | \
    cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: Could not find release asset for $ASSET_PATTERN"
    exit 1
fi

INSTALL_DIR="$HOME/.local/share/bob"
BIN_DIR="$HOME/.local/bin"
ZIP_FILE="/tmp/bob_install.zip"

echo "Downloading from $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "$ZIP_FILE"

TEMP_EXTRACT="/tmp/bob_extract_$$"
mkdir -p "$TEMP_EXTRACT"

echo "Installing..."
unzip -q "$ZIP_FILE" -d "$TEMP_EXTRACT"

BOB_BIN=$(find "$TEMP_EXTRACT" -type f -name "bob" | head -n 1)

if [ -z "$BOB_BIN" ]; then
    echo "Error: Could not find 'bob' executable in zip."
    exit 1
fi

SOURCE_DIR=$(dirname "$BOB_BIN")

rm -rf "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
mv "$SOURCE_DIR"/* "$INSTALL_DIR/"

mkdir -p "$BIN_DIR"
ln -sf "$INSTALL_DIR/bob" "$BIN_DIR/bob"

rm "$ZIP_FILE"
rm -rf "$TEMP_EXTRACT"

echo "✅ Bob installed successfully to $BIN_DIR/bob"
# Check if in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "⚠️  Warning: $BIN_DIR is not in your PATH."
    echo "   Add this to your shell config: export PATH=\"\$PATH:$BIN_DIR\""
fi
