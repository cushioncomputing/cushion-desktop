#!/bin/bash

# Simple script to create a DMG from the built .app
set -e

APP_PATH="src-tauri/target/debug/bundle/macos/Cushion Developer.app"
DMG_PATH="src-tauri/target/debug/bundle/dmg/Cushion Developer_0.1.0_aarch64.dmg"
VOLUME_NAME="Cushion Developer"

if [ ! -d "$APP_PATH" ]; then
    echo "Error: $APP_PATH not found. Build the app first with npm run build:dev"
    exit 1
fi

echo "Creating DMG from $APP_PATH..."

# Remove old DMG if exists
rm -f "$DMG_PATH"

# Create DMG directory if it doesn't exist
mkdir -p "$(dirname "$DMG_PATH")"

# Create DMG
hdiutil create -volname "$VOLUME_NAME" -srcfolder "$APP_PATH" -ov -format UDZO "$DMG_PATH"

echo "✅ DMG created at: $DMG_PATH"
