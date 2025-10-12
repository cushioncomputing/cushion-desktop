#!/bin/bash

# Post-build script to create and notarize DMG
# Usage: ./notarize-dmg.sh <app-path> <output-dmg-path> <volume-name>

set -e

APP_PATH="$1"
DMG_PATH="$2"
VOLUME_NAME="$3"

if [ -z "$APP_PATH" ] || [ -z "$DMG_PATH" ] || [ -z "$VOLUME_NAME" ]; then
    echo "Usage: $0 <app-path> <output-dmg-path> <volume-name>"
    exit 1
fi

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ App not found at: $APP_PATH"
    exit 1
fi

echo "📦 Creating DMG from $APP_PATH..."
mkdir -p "$(dirname "$DMG_PATH")"

# Create DMG
hdiutil create -volname "$VOLUME_NAME" \
    -srcfolder "$APP_PATH" \
    -ov -format UDZO \
    "$DMG_PATH"

echo "✅ DMG created at: $DMG_PATH"

# Check if we have notarization credentials
if [ -z "$APPLE_ID" ] || [ -z "$APPLE_PASSWORD" ] || [ -z "$APPLE_TEAM_ID" ]; then
    echo "⚠️  Skipping DMG notarization (no Apple credentials found)"
    exit 0
fi

echo "🔐 Notarizing DMG..."
xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait

echo "📎 Stapling notarization ticket to DMG..."
xcrun stapler staple "$DMG_PATH"

echo "✅ DMG notarization complete!"

# Verify
echo "🔍 Verifying notarization..."
xcrun stapler validate "$DMG_PATH"

echo "🎉 DMG is ready for distribution: $DMG_PATH"
