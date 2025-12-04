#!/usr/bin/env bash
set -e

if ! command -v unzip &> /dev/null; then
    echo "Error: 'unzip' is required but not installed."
    exit 1
fi

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
    aarch64|arm64) ARCH="arm" ;;  # matches asset names bob-macos-arm.zip, bob-linux-arm.zip
    *) echo "Unsupported Architecture: $ARCH"; exit 1 ;;
esac

# Construct the asset name pattern to look for
# Example: bob-linux-x86_64.zip or bob-macos-arm.zip
ASSET_PATTERN="bob-${PLATFORM}-${ARCH}.zip"

echo "Fetching latest release for $PLATFORM-$ARCH..."

# Get the download URL from the GitHub API (no [] around the URL!)
DOWNLOAD_URL=$(
  curl -s https://api.github.com/repos/MordechaiHadad/bob/releases/latest | \
    grep "browser_download_url" | \
    grep "$ASSET_PATTERN" | \
    head -n 1 | \
    cut -d '"' -f 4
)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: Could not find release asset for $ASSET_PATTERN"
    exit 1
fi

INSTALL_DIR="$HOME/.local/share/bob_bin"
BIN_DIR="$HOME/.local/bin"
ZIP_FILE="/tmp/bob_install.zip"
TEMP_EXTRACT="/tmp/bob_extract_$$"

echo "Downloading from $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "$ZIP_FILE"

echo "Installing..."
mkdir -p "$TEMP_EXTRACT"
unzip -q "$ZIP_FILE" -d "$TEMP_EXTRACT"

# FLATTEN: find the 'bob' binary inside the extracted tree
BOB_BIN=$(find "$TEMP_EXTRACT" -type f -name "bob" | head -n 1)

if [ -z "$BOB_BIN" ]; then
    echo "Error: Could not find 'bob' executable in zip."
    rm -rf "$TEMP_EXTRACT" "$ZIP_FILE"
    exit 1
fi

SOURCE_DIR=$(dirname "$BOB_BIN")

# Clean old install and move new files in
rm -rf "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
mv "$SOURCE_DIR"/* "$INSTALL_DIR/"

# Link to bin (for PATH)
mkdir -p "$BIN_DIR"
ln -sf "$INSTALL_DIR/bob" "$BIN_DIR/bob"

# Cleanup
rm -rf "$TEMP_EXTRACT" "$ZIP_FILE"

echo "Bob installed successfully to $BIN_DIR/bob"
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "Warning: $BIN_DIR is not in your PATH."
    echo "Add this to your shell config: export PATH=\"\$PATH:$BIN_DIR\""
fi
