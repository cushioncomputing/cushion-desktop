#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const configPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json');
const backupPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json.backup');

// Read the current config
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

// Check if we're in test mode or dev mode
const isTestMode = process.env.TAURI_BUILD_TEST === 'true';
const isDevMode = process.env.TAURI_BUILD_DEV === 'true';

if (isTestMode) {
  console.log('🧪 Building in test mode - using localhost');

  // Create backup if it doesn't exist
  if (!fs.existsSync(backupPath)) {
    fs.writeFileSync(backupPath, JSON.stringify(config, null, 2));
  }

  // Modify config for test mode
  config.build.frontendDist = 'http://localhost:3000';

  // Write the modified config
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

  console.log('✅ Configuration updated for test build');
} else if (isDevMode) {
  console.log('👨‍💻 Building in dev mode - using localhost with custom app name');

  // Create backup if it doesn't exist
  if (!fs.existsSync(backupPath)) {
    fs.writeFileSync(backupPath, JSON.stringify(config, null, 2));
  }

  // Modify config for dev mode
  config.build.devUrl = 'http://localhost:3000';
  config.build.frontendDist = 'http://localhost:3000';
  config.productName = 'Cushion Developer';
  config.identifier = 'com.cushion.desktop.dev';
  config.bundle.icon = [
    'icons/dev-icon.icns',
    'icons/dev-icon.ico'
  ];
  config.plugins['deep-link'].desktop.schemes = ['cushion-dev'];

  // Dev build checks for latest-dev.json
  // TODO: When ready for public beta, switch to cushion-desktop-updates repo:
  // "https://github.com/cushioncomputing/cushion-desktop-updates/releases/latest/download/latest-dev.json"
  config.plugins.updater.endpoints = [
    'https://github.com/cushioncomputing/cushion-desktop/releases/latest/download/latest-dev.json'
  ];

  // Write the modified config
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

  console.log('✅ Configuration updated for dev build');
} else {
  console.log('🚀 Building in production mode - using app.cushion.so');

  // Create backup if it doesn't exist
  if (!fs.existsSync(backupPath)) {
    fs.writeFileSync(backupPath, JSON.stringify(config, null, 2));
  }

  // Modify config for production mode
  config.build.devUrl = "https://app.cushion.so";
  config.build.frontendDist = "https://app.cushion.so";
  config.productName = 'Cushion';
  config.identifier = 'com.cushion.desktop';
  config.bundle.icon = [
    'icons/icon.icns',
    'icons/icon.ico'
  ];
  config.plugins['deep-link'].desktop.schemes = ['cushion'];

  // Production build checks for latest.json
  // Currently uses private repo (works for internal team with GitHub auth)
  // TODO: When ready for public beta, switch to cushion-desktop-updates repo:
  // "https://github.com/cushioncomputing/cushion-desktop-updates/releases/latest/download/latest.json"
  config.plugins.updater.endpoints = [
    'https://github.com/cushioncomputing/cushion-desktop/releases/latest/download/latest.json'
  ];

  // Write the modified config
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  console.log('✅ Configuration updated for production build');
}