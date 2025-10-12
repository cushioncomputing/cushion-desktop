# Release Workflow Quick Reference

## 🚀 How to Release a New Version

### Option 1: Automated (Recommended)

**No manual version bumping required!**

1. **Create a PR** with your changes
   ```bash
   git checkout -b feature/my-awesome-feature
   # ... make your changes ...
   git push origin feature/my-awesome-feature
   ```

2. **Add a version label** to your PR on GitHub:
   - `version:patch` → Bug fixes (0.1.0 → 0.1.1)
   - `version:minor` → New features (0.1.0 → 0.2.0)
   - `version:major` → Breaking changes (0.1.0 → 1.0.0)

3. **Merge the PR** → Done! GitHub Actions will:
   - ✅ Auto-bump version across all files
   - ✅ Commit the version change
   - ✅ Build both dev and prod versions
   - ✅ Sign and notarize
   - ✅ Create GitHub Release
   - ✅ Users' apps auto-update!

### Option 2: Manual

If you forget to add a label or prefer manual control:

```bash
# Bump version locally
npm run version:patch  # or minor/major

# Commit and push
git add .
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

## ⚠️ Important Notes

### You DON'T Need To:
- ❌ Manually edit version numbers
- ❌ Remember which files to update
- ❌ Manually create GitHub Releases
- ❌ Generate update manifests
- ❌ Sign or notarize builds

### You DO Need To:
- ✅ Add a `version:*` label to your PR (or bump manually)
- ✅ That's it!

## 🔍 Version Sync Check

Every PR automatically checks that versions are synced:

**If versions are out of sync:**
- ✅ **With label**: Check passes (will auto-bump on merge)
- ❌ **Without label**: Check fails (reminds you to add label)

**If versions are already synced:**
- ✅ Check passes regardless of label

## 📋 GitHub Labels

Create these labels in your GitHub repo:

| Label | Description | Version Change |
|-------|-------------|----------------|
| `version:patch` | Bug fixes, small changes | 0.1.0 → 0.1.1 |
| `version:minor` | New features, backwards compatible | 0.1.0 → 0.2.0 |
| `version:major` | Breaking changes | 0.1.0 → 1.0.0 |

**To create labels:**
1. Go to `https://github.com/cushioncomputing/cushion-desktop/labels`
2. Click "New label"
3. Create each of the three labels above

## 🔄 What Happens When You Merge

### 1. Version Bump Workflow Runs
```
PR merged with version:patch label
↓
version-bump.yml triggers
↓
Runs npm run version:patch
↓
Commits: "chore: bump version to X.Y.Z [skip ci]"
↓
Pushes to main
```

### 2. Release Workflow Triggers
```
New commit to main (version bump)
↓
release.yml triggers
↓
Builds prod + dev versions in parallel
↓
Signs and notarizes both
↓
Generates latest.json + latest-dev.json
↓
Creates GitHub Release with all artifacts
↓
Apps check for updates and auto-install
```

## 🎯 Update Channels

| Channel | App Name | Bundle ID | Update File |
|---------|----------|-----------|-------------|
| Production | `Cushion.app` | `com.cushion.desktop` | `latest.json` |
| Development | `Cushion Developer.app` | `com.cushion.desktop.dev` | `latest-dev.json` |

Both apps can run simultaneously on the same machine.

## 🐛 Troubleshooting

### "Version check failed" on PR
**Problem**: Versions are out of sync and no version label

**Solution**: Add a `version:patch`, `version:minor`, or `version:major` label to your PR

---

### Auto-bump didn't work
**Problem**: Version wasn't bumped after merging

**Checks**:
1. Did the PR have a `version:*` label?
2. Was the PR actually merged (not just closed)?
3. Check GitHub Actions tab for errors

---

### Release didn't trigger
**Problem**: No release was created after version bump

**Checks**:
1. Was version bump commit pushed to main?
2. Check if commit message has `[skip ci]` (version bump has this, but it only skips the bump workflow, not release workflow)
3. Check GitHub Actions tab for errors
4. Verify GitHub Secrets are configured

---

### "Permission denied" in workflow
**Problem**: Workflow can't push commits

**Solution**: Ensure workflow has write permissions:
- Uses `GITHUB_TOKEN` (has write access by default)
- Or create a Personal Access Token with repo permissions

## 📖 Examples

### Example 1: Bug Fix Release
```bash
# Create branch
git checkout -b fix/login-issue

# Fix the bug
# ... make changes ...

# Push and create PR
git push origin fix/login-issue

# On GitHub:
# 1. Create PR
# 2. Add label: version:patch
# 3. Get approval
# 4. Merge PR
# → Version auto-bumps to 0.1.1
# → Release created automatically
```

### Example 2: New Feature Release
```bash
# Create branch
git checkout -b feature/dark-mode

# Build the feature
# ... make changes ...

# Push and create PR
git push origin feature/dark-mode

# On GitHub:
# 1. Create PR
# 2. Add label: version:minor
# 3. Get approval
# 4. Merge PR
# → Version auto-bumps to 0.2.0
# → Release created automatically
```

### Example 3: Emergency Hotfix
```bash
# Fix is urgent, skip PR

# Make the fix
git checkout main
git pull

# Manually bump version
npm run version:patch

# Commit and push
git add .
git commit -m "hotfix: critical security patch"
git push origin main

# → Release triggered immediately
```

## 🎉 Benefits

1. **Never forget to bump versions** - Label on PR is visual reminder
2. **Consistent version numbers** - Automated sync across all files
3. **Less manual work** - One label replaces 3-4 manual steps
4. **Fewer mistakes** - Can't merge without version plan
5. **Clear changelog** - PR labels make it obvious what changed

## 🔗 Related Files

- `.github/workflows/version-bump.yml` - Auto-bump workflow
- `.github/workflows/version-check.yml` - Sync validation
- `.github/workflows/release.yml` - Build and publish
- `scripts/sync-version.js` - Version sync script
- `OTA-UPDATES-PLAN.md` - Full implementation details
