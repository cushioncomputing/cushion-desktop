# Cushion Desktop App

A native desktop wrapper for the Cushion web application built with Tauri v2.

## Overview

This Tauri application provides a native desktop experience for Cushion by embedding the existing web application in a native webview. The app supports:

- **Desktop**: macOS, Windows, Linux
- **Magic Link Authentication**: Seamless authentication for both web and desktop users
- **Native Push Notifications**: System-level notifications
- **Deep Link Support**: `cushion://` URL scheme handling

## Architecture

The app uses Tauri's webview to display the existing Cushion web application (`../web-app`) running on `localhost:3001` during development, or served from the built static files in production.

### Key Features

- ✅ Native desktop app shell
- ✅ Magic link authentication support for both web and desktop
- ✅ Native push notifications via Tauri commands
- ✅ Deep link handling (`cushion://` URL scheme)
- ✅ Cross-platform support (macOS, Windows, Linux)
- ✅ Hot reload during development
- ✅ Static build support for production

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

3. **Node.js & Yarn**: Already installed in parent project

### Windows
1. **Visual Studio Build Tools** or **Visual Studio Community**
2. **Rust**: Install via rustup
3. **WebView2**: Usually pre-installed on Windows 10/11

### Linux
1. **Build dependencies**:
   ```bash
   sudo apt update
   sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
   ```
2. **Rust**: Install via rustup

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

This will:
1. Start the Next.js web app on localhost:3001
2. Launch the Tauri desktop app pointing to the web app
3. Enable hot reloading for both frontend and backend changes

### Alternative Development Commands
```bash
# Start desktop app without waiting for web server (if web app is already running)
npm run dev:standalone

# Test deep link handling
npm run test:deep-links
```

## Building for Production

### Desktop App Build
```bash
npm run build
```

This will:
1. Build the Next.js app as a static export with `TAURI_BUILD=true`
2. Bundle the static files into the Tauri app
3. Create platform-specific installers in `src-tauri/target/release/bundle/`

### Build Outputs
- **macOS**: `.dmg` and `.app` files
- **Windows**: `.exe` installer and `.msi`
- **Linux**: `.deb`, `.rpm`, and `.AppImage`

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

Test the deep link functionality:
```bash
# macOS/Linux
open "cushion://auth/success?token=test&callbackUrl=%2Fdashboard"

# Windows
start "cushion://auth/success?token=test&callbackUrl=%2Fdashboard"
```

## Native Features

### Push Notifications
```javascript
// Available via Tauri commands
await invoke('show_notification', {
  title: 'New Message',
  body: 'You have a new message in #general'
});
```

### Desktop Detection
```javascript
// Check if running in desktop app
import { isTauriApp } from '@/lib/tauri';

if (isTauriApp()) {
  // Desktop-specific functionality
}
```

## Configuration

### Main Configuration Files

- `src-tauri/tauri.conf.json` - Main Tauri configuration
- `src-tauri/Cargo.toml` - Rust dependencies and metadata
- `package.json` - NPM scripts and dependencies

### Key Configuration Options

- **Development URL**: Points to `http://localhost:3000` (web-app dev server)
- **Production Build**: Uses `../../web-app/out` (static build output)
- **Window Settings**: 1200x800 default, 800x600 minimum
- **App Identifier**: `com.cushion.desktop`
- **Icons**: Located in `src-tauri/icons/`

## Project Structure

```
cushion-desktop/
├── src-tauri/                 # Rust backend code
│   ├── src/
│   │   ├── main.rs           # Main Tauri application entry
│   │   └── lib.rs            # Library functions
│   ├── icons/                # App icons for all platforms
│   ├── capabilities/         # Tauri permissions
│   ├── gen/                  # Generated platform code
│   │   └── apple/            # iOS Xcode project
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── src/                      # Frontend placeholder (unused)
└── package.json             # NPM configuration
```

## Integration with Web App

The desktop app integrates with the main Cushion web application by:

1. **Development**: Starting the web-app dev server (`npm run dev`) and connecting to `localhost:3000`
2. **Production**: Serving static files from the web-app build output
3. **APIs**: All API calls go through the same endpoints as the web version
4. **Authentication**: Uses the same NextAuth.js authentication flow

## iOS Specific Features

- Native iOS webview with Safari engine
- Touch-optimized interface automatically inherited from responsive web design
- iOS system integration (notifications, share sheet, etc.)
- App Store distribution ready

## Distribution

### Desktop

- **macOS**: .dmg installer
- **Windows**: .msi installer  
- **Linux**: .deb and .rpm packages

### iOS

- App Store distribution through Xcode
- TestFlight beta testing
- Enterprise distribution (if applicable)

## Development Workflow

1. Make changes to the web application (`../web-app`)
2. Test in desktop app with `npm run dev:desktop`
3. Test in iOS simulator with `npm run dev:ios`
4. Build for distribution with `npm run build:desktop` or `npm run build:ios`

## Troubleshooting

### Common Issues

- **Port conflicts**: Ensure web-app is running on port 3000
- **Build failures**: Check that web-app builds successfully first
- **iOS simulator issues**: Restart simulator or use `xcrun simctl list devices`

### Development Tips

- Use Safari Developer Tools for debugging the iOS webview
- Desktop webview debugging available through system browser dev tools
- Hot reload works for both desktop and mobile during development

## Contributing

This is part of the larger Cushion project. Follow the same development practices as the main web application.