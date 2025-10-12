# OTA Updates Implementation Plan

## Overview
Implementing fully functional OTA updates for internal testing using private repo GitHub Releases. This will work for team members with GitHub authentication. When ready for public beta, we'll migrate to a public `cushion-desktop-updates` repository.

## Architecture

### Current Setup (Internal Testing)
- **Update Host**: Private repo GitHub Releases (`cushion-desktop`)
- **Works For**: Team members with GitHub authentication
- **Dev Channel**: `latest-dev.json`
- **Prod Channel**: `latest.json`

### Future Setup (Public Beta)
- **Update Host**: Public repo GitHub Releases (`cushion-desktop-updates`)
- **Works For**: Anyone (no authentication required)
- Same channel structure

## Implementation Checklist

### ✅ Phase 1: Generate Signing Keys
- [x] Generate Tauri update signing keypair
- [x] Store private key in `.env`
- [x] Add public key to `tauri.conf.json`
- [x] Add signing keys to `.gitignore`

### ✅ Phase 2: Install Dependencies
- [x] Add `tauri-plugin-updater` to `Cargo.toml`
- [x] Add `tauri-plugin-dialog` to `Cargo.toml`
- [x] Install `@tauri-apps/plugin-updater` via npm
- [x] Install `@tauri-apps/plugin-dialog` via npm

### ✅ Phase 3: Create Version Management
- [x] Create `scripts/sync-version.js` for version synchronization
- [x] Add npm scripts: `version:major`, `version:minor`, `version:patch`
- [x] Validates versions across package.json, Cargo.toml, tauri.conf.json

### ✅ Phase 4: Implement Updater Commands
- [x] Create `src-tauri/src/commands/updater.rs`
  - `check_for_updates()` - Check if update available
  - `install_update()` - Download and install update
  - `get_app_version()` - Get current version
- [x] Export updater module in `commands/mod.rs`

### ✅ Phase 5: Initialize Plugins
- [x] Add `tauri_plugin_updater` to `lib.rs`
- [x] Add `tauri_plugin_dialog` to `lib.rs`
- [x] Register updater commands in `invoke_handler!`

### ✅ Phase 6: Configure Permissions
- [x] Add `updater:default` permission to `capabilities/default.json`
- [x] Add `updater:allow-check` permission
- [x] Add `dialog:default` permission

### ✅ Phase 7: Configure Updater
- [x] Add updater config to `tauri.conf.json`:
  - Set `active: true`
  - Set endpoint to private repo
  - Add public key
- [x] Update `build-config.js` to set different endpoints per build type:
  - Dev: `latest-dev.json`
  - Prod: `latest.json`

### 🔄 Phase 8: GitHub Actions Workflow (IN PROGRESS)
- [ ] Create `.github/workflows/release.yml`
- [ ] Configure to trigger on push to `main`
- [ ] Build both dev and prod versions
- [ ] Sign and notarize macOS apps
- [ ] Generate update manifests (`latest.json`, `latest-dev.json`)
- [ ] Create GitHub Release with artifacts
- [ ] Include migration comments for public beta

### ⏳ Phase 9: Documentation
- [ ] Update README with releases section
- [ ] Document version bump workflow
- [ ] Explain dev vs prod update channels
- [ ] Add migration guide for public beta
- [ ] Create `.env.example` template

### ⏳ Phase 10: GitHub Secrets Setup
Required secrets (to be added via GitHub UI):
- `APPLE_ID` - From `.env`
- `APPLE_PASSWORD` - From `.env`
- `APPLE_TEAM_ID` - From `.env`
- `TAURI_SIGNING_PRIVATE_KEY` - Content of `.tauri-signing-key.txt`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - Password used during key generation

## Workflow After Setup

### To Release a New Version (Automated):
1. Create PR with your changes
2. Add version label to PR:
   - `version:patch` - Bug fixes (0.1.0 → 0.1.1)
   - `version:minor` - New features (0.1.0 → 0.2.0)
   - `version:major` - Breaking changes (0.1.0 → 1.0.0)
3. Merge PR
4. GitHub Actions automatically:
   - Bumps version across all files
   - Commits version change
   - Builds dev and prod versions
   - Signs and notarizes
   - Creates GitHub Release
   - Generates update manifests
5. Team members' apps auto-update within minutes

### To Release a New Version (Manual):
1. Run `npm run version:patch` (or minor/major)
2. Commit version changes
3. Push to `main` branch
4. GitHub Actions automatically handles the rest

### Version Sync Protection:
- PR checks ensure versions are synced before merging
- Prevents accidental version mismatches
- Reminds you to add version label if versions are out of sync

### Update Channels:
- **Dev Channel**: `Cushion Developer.app` checks `latest-dev.json`
- **Prod Channel**: `Cushion.app` checks `latest.json`

## Migration to Public Beta

When ready for external beta testing:

1. **Create Public Repo**: `cushioncomputing/cushion-desktop-updates`
2. **Update Endpoints** in `build-config.js`:
   ```javascript
   // Change from:
   'https://github.com/cushioncomputing/cushion-desktop/releases/...'
   // To:
   'https://github.com/cushioncomputing/cushion-desktop-updates/releases/...'
   ```
3. **Update Workflow**: Modify `.github/workflows/release.yml` to publish to new repo
4. **Release New Version**: Build with updated endpoints
5. **Archive Old Repo**: Once all users updated

## File Structure

```
cushion-desktop/
├── .env                              # Signing keys & Apple credentials (gitignored)
├── .tauri-signing-key.txt           # Private key (gitignored)
├── .tauri-signing-key.txt.pub       # Public key (gitignored)
├── .github/
│   └── workflows/
│       └── release.yml              # CI/CD workflow
├── scripts/
│   └── sync-version.js              # Version management
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   │   ├── updater.rs          # Update commands
│   │   │   └── mod.rs              # Export updater
│   │   └── lib.rs                  # Initialize plugins
│   ├── capabilities/
│   │   └── default.json            # Updater permissions
│   └── tauri.conf.json             # Updater config
├── build-config.js                  # Configure update endpoints
└── package.json                     # Version scripts

```

## Important Notes

### Security
- Private key MUST stay secure (never commit)
- Public key is safe to commit
- Update manifests are signed with private key
- Apps verify signatures with public key

### Private Repo Limitation
- Currently using private repo for internal testing
- Works for team members (GitHub authenticated)
- External users CANNOT download updates
- Must migrate to public repo before external beta

### Version Consistency
- MUST keep versions in sync across 3 files:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `src-tauri/tauri.conf.json`
- Use `npm run version:patch` to bump safely

## Testing Checklist

Before going live:
- [ ] Test version bump scripts
- [ ] Test GitHub Actions workflow
- [ ] Verify signing keys work
- [ ] Test dev channel updates
- [ ] Test prod channel updates
- [ ] Verify both apps can coexist
- [ ] Test update notifications
- [ ] Test update installation
- [ ] Verify notarization passes

## Troubleshooting

### Updates Not Found
- Check GitHub Release exists
- Verify manifest file uploaded
- Check endpoint URL in tauri.conf.json
- Verify app can reach GitHub

### Signature Verification Failed
- Public key in tauri.conf.json matches private key
- Private key used during build matches
- Manifest was signed correctly

### Permission Denied (Private Repo)
- User not authenticated with GitHub
- Need to migrate to public updates repo
- Or distribute via other means

## Current Status

**Completed:**
- ✅ All updater code implemented
- ✅ Signing keys generated
- ✅ Configuration complete
- ✅ Version management ready

**In Progress:**
- 🔄 GitHub Actions workflow
- 🔄 Documentation updates

**Next Steps:**
1. Complete GitHub Actions workflow
2. Add GitHub secrets
3. Test first release
4. Verify updates work internally
