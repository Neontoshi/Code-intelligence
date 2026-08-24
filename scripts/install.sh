#!/bin/bash
# Auto-detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-x86_64)  ASSET="ci_linux_x86_64" ;;
  linux-aarch64) ASSET="ci_linux_arm64" ;;
  darwin-x86_64) ASSET="ci_macos_intel" ;;
  darwin-arm64)  ASSET="ci_macos_arm64" ;;
  *) echo "Unsupported: $OS $ARCH"; exit 1 ;;
esac

curl -fsSL -o ci "https://github.com/neontoshi/Code-intelligence/releases/latest/download/$ASSET"
chmod +x ci
sudo mv ci /usr/local/bin/
echo "✅ Installed ci version: $(ci --version)"
