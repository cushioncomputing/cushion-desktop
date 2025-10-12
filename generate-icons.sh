#!/bin/bash

# Script to generate .icns files from a 1024x1024 source image
# Usage: ./generate-icons.sh

set -e

# Ask for source image path
echo "Enter the path to your 1024x1024 source image:"
read -p "Path: " SOURCE_IMAGE

# Expand tilde to home directory
SOURCE_IMAGE="${SOURCE_IMAGE/#\~/$HOME}"

# Ask if it's a dev or prod icon
echo ""
echo "Is this a dev or prod icon?"
read -p "Enter 'dev' or 'prod': " ICON_TYPE

if [ "$ICON_TYPE" = "dev" ]; then
    OUTPUT_NAME="dev-icon"
    echo "🔧 Generating dev icon"
elif [ "$ICON_TYPE" = "prod" ]; then
    OUTPUT_NAME="icon"
    echo "🚀 Generating prod icon"
else
    echo "Error: Invalid option. Please enter 'dev' or 'prod'"
    exit 1
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICONS_DIR="$SCRIPT_DIR/src-tauri/icons"
TEMP_DIR=$(mktemp -d)

# Check if source image exists
if [ ! -f "$SOURCE_IMAGE" ]; then
    echo "Error: Source image '$SOURCE_IMAGE' not found"
    exit 1
fi

echo "🎨 Generating icon from: $SOURCE_IMAGE"
echo "📁 Output: $ICONS_DIR/${OUTPUT_NAME}.icns"

# Create iconset directory
ICONSET="$TEMP_DIR/${OUTPUT_NAME}.iconset"
mkdir -p "$ICONSET"

# Generate all required sizes
sips -z 16 16     "$SOURCE_IMAGE" --out "$ICONSET/icon_16x16.png" > /dev/null
sips -z 32 32     "$SOURCE_IMAGE" --out "$ICONSET/icon_16x16@2x.png" > /dev/null
sips -z 32 32     "$SOURCE_IMAGE" --out "$ICONSET/icon_32x32.png" > /dev/null
sips -z 64 64     "$SOURCE_IMAGE" --out "$ICONSET/icon_32x32@2x.png" > /dev/null
sips -z 128 128   "$SOURCE_IMAGE" --out "$ICONSET/icon_128x128.png" > /dev/null
sips -z 256 256   "$SOURCE_IMAGE" --out "$ICONSET/icon_128x128@2x.png" > /dev/null
sips -z 256 256   "$SOURCE_IMAGE" --out "$ICONSET/icon_256x256.png" > /dev/null
sips -z 512 512   "$SOURCE_IMAGE" --out "$ICONSET/icon_256x256@2x.png" > /dev/null
sips -z 512 512   "$SOURCE_IMAGE" --out "$ICONSET/icon_512x512.png" > /dev/null
sips -z 1024 1024 "$SOURCE_IMAGE" --out "$ICONSET/icon_512x512@2x.png" > /dev/null

# Convert to .icns
iconutil -c icns "$ICONSET" -o "$ICONS_DIR/${OUTPUT_NAME}.icns"

# Cleanup
rm -rf "$TEMP_DIR"

echo "✅ Successfully generated $ICONS_DIR/${OUTPUT_NAME}.icns"
