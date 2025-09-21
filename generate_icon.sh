#!/bin/bash

# Script to generate .icns file from PNG source for Tauri app
set -e

# Source and destination paths
SOURCE_PNG="/Users/davidhawkins/Downloads/Icon-iOS-Default-1024x1024@1x.png"
DEST_DIR="/Users/davidhawkins/Developer/cushion/cushion-desktop/src-tauri/icons"
ICONSET_DIR="/tmp/cushion.iconset"

# Check if source file exists
if [ ! -f "$SOURCE_PNG" ]; then
    echo "Error: Source PNG file not found at $SOURCE_PNG"
    exit 1
fi

echo "Creating iconset directory..."
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

# Generate all required icon sizes for .icns
echo "Generating icon sizes..."

# Standard sizes for .icns
sips -z 16 16 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16.png"
sips -z 32 32 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png"
sips -z 32 32 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32.png"
sips -z 64 64 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png"
sips -z 128 128 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128.png"
sips -z 256 256 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png"
sips -z 256 256 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256.png"
sips -z 512 512 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png"
sips -z 512 512 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512.png"
sips -z 1024 1024 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512@2x.png"

echo "Creating .icns file..."
iconutil -c icns "$ICONSET_DIR" -o "$DEST_DIR/icon.icns"

echo "Cleaning up temporary files..."
rm -rf "$ICONSET_DIR"

echo "✅ Successfully created new icon.icns file at $DEST_DIR/icon.icns"
echo "Original icon has been replaced with the new one from $SOURCE_PNG"