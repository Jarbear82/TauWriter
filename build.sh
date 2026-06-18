#!/usr/bin/env bash
set -e

# 1. Define color constants for beautiful log tracking
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}[1/3]${NC} Cleaning old local Linux binary target..."
mkdir -p extension/bin
rm -f extension/bin/tauwriter-lsp-linux-x64

echo -e "${BLUE}[2/3]${NC} Compiling local LSP sub-project in Release mode..."
cargo build --release --manifest-path lsp/Cargo.toml

echo -e "${BLUE}[3/3]${NC} Moving build artifact to dev extension bin directory..."
cp target/release/tauwriter-lsp extension/bin/tauwriter-lsp-linux-x64
chmod +x extension/bin/tauwriter-lsp-linux-x64

# NEW: Also copy to Zed's work directory so the extension can find it immediately
ZED_WORK_DIR="$HOME/.local/share/zed/extensions/work/tauwriter/bin"
echo -e "${BLUE}[Bonus]${NC} Syncing to Zed work directory: $ZED_WORK_DIR"
mkdir -p "$ZED_WORK_DIR"
cp target/release/tauwriter-lsp "$ZED_WORK_DIR/tauwriter-lsp-linux-x64"
chmod +x "$ZED_WORK_DIR/tauwriter-lsp-linux-x64"

echo -e "${GREEN}✓ Local Linux LSP environment successfully updated!${NC}"
echo -e "You can reload your Zed window to test changes instantly."
