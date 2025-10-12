#!/usr/bin/env node

/**
 * Version Sync Script
 *
 * Synchronizes version across package.json, Cargo.toml, and tauri.conf.json
 * Usage: node scripts/sync-version.js [major|minor|patch|<version>]
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.join(__dirname, '..');

const PACKAGE_JSON = path.join(rootDir, 'package.json');
const CARGO_TOML = path.join(rootDir, 'src-tauri', 'Cargo.toml');
const TAURI_CONF = path.join(rootDir, 'src-tauri', 'tauri.conf.json');

function readVersion(file) {
  if (file.endsWith('.json')) {
    const content = JSON.parse(fs.readFileSync(file, 'utf8'));
    return content.version;
  } else if (file.endsWith('.toml')) {
    const content = fs.readFileSync(file, 'utf8');
    const match = content.match(/^version\s*=\s*"([^"]+)"/m);
    return match ? match[1] : null;
  }
}

function writeVersion(file, version) {
  if (file.endsWith('.json')) {
    const content = JSON.parse(fs.readFileSync(file, 'utf8'));
    content.version = version;
    fs.writeFileSync(file, JSON.stringify(content, null, 2) + '\n');
  } else if (file.endsWith('.toml')) {
    let content = fs.readFileSync(file, 'utf8');
    content = content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
    fs.writeFileSync(file, content);
  }
}

function bumpVersion(current, type) {
  const parts = current.split('.').map(Number);

  switch (type) {
    case 'major':
      return `${parts[0] + 1}.0.0`;
    case 'minor':
      return `${parts[0]}.${parts[1] + 1}.0`;
    case 'patch':
      return `${parts[0]}.${parts[1]}.${parts[2] + 1}`;
    default:
      // Assume it's a specific version
      return type;
  }
}

function validateVersion(version) {
  return /^\d+\.\d+\.\d+$/.test(version);
}

function main() {
  const arg = process.argv[2];

  if (!arg) {
    console.log('Usage: node scripts/sync-version.js [major|minor|patch|<version>]');
    console.log('\nCurrent versions:');
    console.log(`  package.json:      ${readVersion(PACKAGE_JSON)}`);
    console.log(`  Cargo.toml:        ${readVersion(CARGO_TOML)}`);
    console.log(`  tauri.conf.json:   ${readVersion(TAURI_CONF)}`);
    process.exit(0);
  }

  // Read current version from package.json
  const currentVersion = readVersion(PACKAGE_JSON);
  const newVersion = ['major', 'minor', 'patch'].includes(arg)
    ? bumpVersion(currentVersion, arg)
    : arg;

  if (!validateVersion(newVersion)) {
    console.error(`❌ Invalid version format: ${newVersion}`);
    console.error('   Version must be in format: X.Y.Z');
    process.exit(1);
  }

  console.log(`📦 Updating version: ${currentVersion} → ${newVersion}`);

  // Update all files
  writeVersion(PACKAGE_JSON, newVersion);
  writeVersion(CARGO_TOML, newVersion);
  writeVersion(TAURI_CONF, newVersion);

  console.log('✅ Version synced across all files:');
  console.log(`   - package.json`);
  console.log(`   - src-tauri/Cargo.toml`);
  console.log(`   - src-tauri/tauri.conf.json`);
  console.log(`\n💡 Next steps:`);
  console.log(`   1. git add .`);
  console.log(`   2. git commit -m "Bump version to ${newVersion}"`);
  console.log(`   3. git push`);
  console.log(`   4. GitHub Actions will build and release automatically`);
}

main();
