# Cushion Desktop App

A native desktop wrapper for the Cushion web application built with Tauri v2.

## Overview

This Tauri application provides a native desktop experience for Cushion by embedding the existing web application in a native webview. The app supports:

- **macOS Desktop**: Native macOS application
- **iOS**: Native iOS application (via Tauri mobile)
- **Magic Link Authentication**: Seamless authentication for both web and desktop users
- **Native Push Notifications**: System-level notifications
- **Deep Link Support**: `cushion://` URL scheme handling

## Architecture

The app uses Tauri's webview to display the existing Cushion web application (`../web-app`) running on `localhost:3000` during development. Production builds are configured dynamically by `build-config.js`.

### Key Features

- ✅ Native macOS and iOS app shell
- ✅ Magic link authentication support for both web and desktop
- ✅ Native push notifications via Tauri commands
- ✅ Deep link handling (`cushion://` URL scheme)
- ✅ Hot reload during development
- ✅ Multiple build modes for development and production

## Prerequisites

### macOS

1. **Xcode Command Line Tools**: Required for native compilation

   ```bash
   xcode-select --install
   sudo xcodebuild -license accept
   ```

2. **Rust**: Install via rustup

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Node.js**: Already installed in parent project

### iOS Development (Optional)

1. **Xcode**: Install from Mac App Store
2. **iOS Simulator**: Included with Xcode
3. **Tauri iOS dependencies**: Automatically set up with `npm run setup:ios`

## Icon Generation

The project includes a script to generate `.icns` files from a 1024x1024 source image for both production and development builds.

### Generate Icons

```bash
./generate-icons.sh
```

The script will:
1. Prompt for the path to your 1024x1024 source image
2. Ask whether to generate a `dev` or `prod` icon
3. Generate all required icon sizes (16x16 through 1024x1024, including @2x variants)
4. Create the `.icns` file in `src-tauri/icons/`

**Output files:**
- `icon.icns` - Production icon (used in release builds)
- `dev-icon.icns` - Development icon (used in dev builds, typically with a visual indicator)

**Note**: Make sure your source image is exactly 1024x1024 pixels for best results.

## Development

### Setup

1. **Install dependencies:**

   ```bash
   npm install
   ```

2. **Initialize iOS support (if needed):**
   ```bash
   npm run setup:ios
   ```

### Start Development Server

```bash
# From cushion-desktop directory
npm run dev
```

This will launch the Tauri desktop app in development mode with hot reloading.

**Note**: Ensure the Next.js web app is running separately on `localhost:3000` (from `../web-app` run `yarn dev`)

### Alternative Development Commands

```bash
# Start desktop app (same as npm run dev)
npm run dev:desktop

# Start iOS simulator
npm run dev:ios

# Test deep link handling
npm run test:deep-links
```

## Building for Production

### Production Build

```bash
npm run build
# or
npm run build:desktop
```

This will:

1. Run `build-config.js` to configure the Tauri build
2. Build the Tauri app in release mode
3. Create platform-specific installers in `src-tauri/target/release/bundle/`

### Development Builds

For faster testing and development builds:

```bash
# Quick dev build (debug mode, .app bundle only, creates .dmg)
npm run build:dev

# Test build (debug mode, preserves dev server URL for testing)
npm run build:test
```

- `build:dev` - Creates a debug build with only the `.app` bundle and wraps it in a `.dmg` for quick testing
- `build:test` - Creates a test build that points to `localhost:3000` for debugging production builds

### iOS Build

```bash
npm run build:ios
```

### Build Outputs

- **macOS**: `.dmg` and `.app` files in `src-tauri/target/release/bundle/macos/`
- **iOS**: `.ipa` file for App Store distribution (via Xcode)

## Authentication Flow

### Magic Link Support

The desktop app fully supports magic link authentication:

1. **Desktop users** get magic links that redirect to `cushion://auth/success`
2. **Web users** get normal magic links that redirect to the web app
3. **Automatic detection** based on client context

### How It Works

1. User clicks "Sign in with email" in desktop app
2. Email is sent with desktop-specific magic link
3. User clicks link in email
4. Link opens desktop app via deep linking
5. App completes authentication and navigates to dashboard

## Deep Link Testing

Test the deep link functionality on macOS:

```bash
npm run test:deep-links

# Or manually:
open "cushion://auth/success?token=test&callbackUrl=%2Fdashboard"
```

## Native Features

### Push Notifications

```javascript
// Available via Tauri commands
await invoke("show_notification", {
  title: "New Message",
  body: "You have a new message in #general",
});
```

### Desktop & Mobile Detection

```javascript
// Check if running in Tauri app (desktop or mobile)
import { isTauriApp } from "@/lib/tauri";

if (isTauriApp()) {
  // Native app-specific functionality
}
```

## Configuration

### Main Configuration Files

- `src-tauri/tauri.conf.json` - Main Tauri configuration
- `src-tauri/Cargo.toml` - Rust dependencies and metadata
- `package.json` - NPM scripts and dependencies

### Key Configuration Options

- **Development URL**: Points to `http://localhost:3000` (web-app dev server)
- **Production Build**: Dynamically configured via `build-config.js`
- **Window Settings**: Configured in `tauri.conf.json`
- **App Identifier**: `com.cushion.desktop`
- **Icons**: Located in `src-tauri/icons/`
- **Build Modes**:
  - Default: Production build with optimized settings
  - `TAURI_BUILD_DEV=true`: Quick debug build for development
  - `TAURI_BUILD_TEST=true`: Test build pointing to localhost

## Project Structure

```
cushion-desktop/
├── src-tauri/                 # Rust backend code
│   ├── src/
│   │   ├── main.rs           # Main Tauri application entry
│   │   └── lib.rs            # Library functions
│   ├── icons/                # App icons (.icns files)
│   │   ├── icon.icns         # Production icon
│   │   └── dev-icon.icns     # Development icon
│   ├── capabilities/         # Tauri permissions
│   ├── gen/                  # Generated platform code
│   │   └── apple/            # iOS Xcode project
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── build-config.js           # Dynamic build configuration
├── create-dev-dmg.sh         # Creates .dmg from .app bundle
├── generate-icons.sh         # Icon generation script
├── src/                      # Frontend placeholder (unused)
└── package.json             # NPM scripts and dependencies
```

## Integration with Web App

The desktop app integrates with the main Cushion web application by:

1. **Development**: Connecting to the web-app dev server at `localhost:3000` (start with `npm run dev` in `../web-app`)
2. **Production**: Configuration determined by `build-config.js` at build time
3. **APIs**: All API calls go through the same endpoints as the web version
4. **Authentication**: Uses the same NextAuth.js authentication flow

### Build Configuration

The `build-config.js` script dynamically configures Tauri builds based on environment variables:

- Reads from `../web-app/package.json` for dependencies and configuration
- Adjusts build settings based on `TAURI_BUILD_TEST` or `TAURI_BUILD_DEV` flags
- Manages production vs. development URLs
- Creates separate apps with different bundle IDs:
  - **Dev**: `com.cushion.desktop.dev` with `cushion-dev://` deep links
  - **Prod**: `com.cushion.desktop` with `cushion://` deep links

## iOS Specific Features

- Native iOS webview with Safari engine
- Touch-optimized interface automatically inherited from responsive web design
- iOS system integration (notifications, share sheet, etc.)
- App Store distribution ready

## Distribution

### macOS

- **.dmg installer**: Universal app bundle for direct download
- **App Store**: (Future) Distribution through Mac App Store

### iOS

- **App Store**: Distribution through Apple App Store
- **TestFlight**: Beta testing platform
- **Ad Hoc**: Development and testing builds

## Development Workflow

1. Start the web app dev server from `../web-app`:

   ```bash
   cd ../web-app && npm run dev
   ```

2. Start the desktop app:

   ```bash
   npm run dev
   ```

3. Make changes to the web application and test in real-time

4. For quick testing of builds:

   ```bash
   npm run build:dev  # Creates a .dmg for quick testing
   ```

5. For production builds:

   ```bash
   npm run build  # Full production build
   ```

6. For iOS development:
   ```bash
   npm run dev:ios  # iOS simulator
   npm run build:ios  # iOS production build
   ```

## Troubleshooting

### Common Issues

- **Port conflicts**: Ensure web-app is running on port 3000
- **Build failures**:
  - Check that `build-config.js` runs successfully
  - Verify `../web-app/package.json` exists and is valid
  - For test builds, ensure `localhost:3000` is accessible
- **iOS simulator issues**: Restart simulator or use `xcrun simctl list devices`
- **Dev build DMG creation**: Ensure `create-dev-dmg.sh` has execute permissions

### Development Tips

- **iOS debugging**: Use Safari Developer Tools (Safari → Develop → Simulator)
- **macOS debugging**: Right-click in app and select "Inspect Element"
- **Hot reload**: Works automatically for both desktop and mobile during development
- **Build issues**: Run `cargo clean` in `src-tauri/` directory if you encounter strange build errors

## Contributing

This is part of the larger Cushion project. Follow the same development practices as the main web application.
