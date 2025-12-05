# Version Bump & Release Process

## Quick Start

To release a new version:

```bash
npm run version:patch   # 0.4.10 → 0.4.11
git add .
git commit -m "chore: bump version to $(node -p "require('./package.json').version")"
git push
```

The GitHub Actions release workflow will automatically build, sign, notarize, and publish the release.

## Version Commands

| Command | Effect | Example |
|---------|--------|---------|
| `npm run version:patch` | Increment patch version | 0.4.10 → 0.4.11 |
| `npm run version:minor` | Increment minor version | 0.4.10 → 0.5.0 |
| `npm run version:major` | Increment major version | 0.4.10 → 1.0.0 |
| `npm run version:check` | Show current versions | Displays all version numbers |

## Files Updated

The version sync script (`scripts/sync-version.js`) updates these files:

- `package.json` - npm package version
- `src-tauri/Cargo.toml` - Rust crate version
- `src-tauri/tauri.conf.json` - Tauri app version

## Release Pipeline

When you push to `main`:

1. **`release.yml`** triggers automatically
2. Builds production app (signed & notarized)
3. Builds dev app (signed & notarized)
4. Creates GitHub Release with tag `vX.Y.Z`
5. Uploads:
   - `Cushion_X.Y.Z_aarch64.dmg` - Production installer
   - `Cushion_Developer_X.Y.Z_aarch64.dmg` - Dev installer
   - `latest.json` - Auto-update manifest (production)
   - `latest-dev.json` - Auto-update manifest (dev)
   - `.tar.gz` and `.sig` files for auto-updater

## Auto-Updates

Users with the app installed will automatically receive updates:
- Production apps check `latest.json`
- Dev apps check `latest-dev.json`

The Tauri updater plugin handles the update flow.

## Commit Message Convention

Use this format for version bump commits:

```
chore: bump version to X.Y.Z
```

This keeps the git history clean and consistent.
