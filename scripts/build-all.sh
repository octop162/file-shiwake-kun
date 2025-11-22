#!/bin/bash
# Build script for all platforms (Unix-based systems)

set -e

echo "🚀 Building ファイル仕訳け君 for all platforms..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if running on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}📦 Building for macOS (Intel)...${NC}"
    npm run tauri:build:macos
    
    echo -e "${BLUE}📦 Building for macOS (Apple Silicon)...${NC}"
    npm run tauri:build:macos-arm
    
    echo -e "${GREEN}✅ macOS builds completed!${NC}"
    echo "Artifacts location:"
    echo "  - src-tauri/target/x86_64-apple-darwin/release/bundle/"
    echo "  - src-tauri/target/aarch64-apple-darwin/release/bundle/"
    
# Check if running on Linux
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${BLUE}📦 Building for Linux...${NC}"
    npm run tauri:build:linux
    
    echo -e "${GREEN}✅ Linux build completed!${NC}"
    echo "Artifacts location:"
    echo "  - src-tauri/target/release/bundle/deb/"
    echo "  - src-tauri/target/release/bundle/appimage/"
    
else
    echo -e "${RED}❌ Unsupported platform: $OSTYPE${NC}"
    echo "Please use build-all.ps1 on Windows"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Build process completed successfully!${NC}"
