#!/bin/bash

# Production build script with automatic DMG creation and notarization
set -e

echo "🚀 Starting production build..."

# Build and notarize the app
node build-config.js
tauri build --bundles app

# Create and notarize DMG
APP_PATH="src-tauri/target/release/bundle/macos/Cushion.app"
DMG_PATH="src-tauri/target/release/bundle/dmg/Cushion_0.1.0_aarch64.dmg"

./notarize-dmg.sh "$APP_PATH" "$DMG_PATH" "Cushion"

echo "🎉 Production build complete!"
echo "📦 App: $APP_PATH"
echo "💿 DMG: $DMG_PATH"
