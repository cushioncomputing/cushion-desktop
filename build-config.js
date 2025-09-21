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

// Check if we're in test mode
const isTestMode = process.env.TAURI_BUILD_TEST === 'true';

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
} else {
  console.log('🚀 Building in production mode - using app.cushion.so');

  // Restore from backup if it exists
  if (fs.existsSync(backupPath)) {
    const backupConfig = fs.readFileSync(backupPath, 'utf8');
    fs.writeFileSync(configPath, backupConfig);
    console.log('✅ Configuration restored from backup');
  }
}