#!/bin/bash
set -e

echo "🔍 Detecting your system..."
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-x86_64)  ASSET="ci" ;;
  linux-aarch64) ASSET="ci" ;;
  darwin-x86_64) ASSET="ci_macos_intel" ;;
  darwin-arm64)  ASSET="ci_macos_arm64" ;;
  *) echo "❌ Unsupported: $OS $ARCH"; exit 1 ;;
esac

echo "📥 Downloading $ASSET..."
curl -fsSL -o ci "https://github.com/neontoshi/Code-intelligence/releases/latest/download/$ASSET"

chmod +x ci

if [ "$OS" = "darwin" ] || [ "$OS" = "linux" ]; then
  sudo mv ci /usr/local/bin/
  echo "✅ Installed ci to /usr/local/bin/"
else
  echo "✅ Downloaded ci.exe to current directory"
  echo "   Move it to a folder in your PATH"
fi

echo "✅ Installation complete!"
ci --version
