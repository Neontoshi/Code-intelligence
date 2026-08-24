#!/bin/bash
set -euo pipefail

echo "🔍 Detecting your system..."
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  linux-x86_64)   ASSET="ci" ;;
  darwin-x86_64)  ASSET="ci_macos_intel" ;;
  darwin-arm64)   ASSET="ci_macos_arm64" ;;
  *)
    echo "❌ Unsupported architecture/OS: $OS $ARCH"
    exit 1
    ;;
esac

DOWNLOAD_URL="https://github.com/neontoshi/Code-intelligence/releases/latest/download/$ASSET"
INSTALL_DIR="/usr/local/bin"
TEMP_FILE="$(mktemp)"

echo "📥 Downloading $ASSET from GitHub..."
curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"

chmod +x "$TEMP_FILE"

# Remove quarantine attribute on macOS
if [ "$OS" = "darwin" ]; then
  xattr -d com.apple.quarantine "$TEMP_FILE" 2>/dev/null || true
fi

echo "📦 Installing to $INSTALL_DIR/ci..."
if [ -w "$INSTALL_DIR" ]; then
  mv "$TEMP_FILE" "$INSTALL_DIR/ci"
else
  sudo mv "$TEMP_FILE" "$INSTALL_DIR/ci"
fi

echo "✅ Installation complete!"
if command -v ci &>/dev/null; then
  ci --version
else
  "$INSTALL_DIR/ci" --version
  echo "⚠️ Make sure $INSTALL_DIR is in your PATH."
fi
