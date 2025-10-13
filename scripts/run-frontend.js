#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Usage: node scripts/run-frontend.js <script> [-- ...args]');
  process.exit(1);
}

const script = args[0];
const separatorIndex = args.indexOf('--');
const scriptArgs = separatorIndex === -1 ? [] : args.slice(separatorIndex + 1);

const npmArgs = ['run', script];
if (scriptArgs.length > 0) {
  npmArgs.push('--', ...scriptArgs);
}

const tauriConfigDir = process.env.TAURI_CONFIG_DIR
  ? path.resolve(process.env.TAURI_CONFIG_DIR)
  : path.resolve(__dirname, '..', 'src-tauri');
const projectRoot = path.resolve(tauriConfigDir, '..');
const frontendDir =
  process.env.LAYERCAKE_FRONTEND_DIR || path.resolve(projectRoot, 'frontend');

let command;
let commandArgs;
if (process.env.npm_execpath) {
  command = process.execPath;
  commandArgs = [process.env.npm_execpath, ...npmArgs];
} else {
  command = process.platform === 'win32' ? 'npm.cmd' : 'npm';
  commandArgs = npmArgs;
}

const result = spawnSync(command, commandArgs, {
  cwd: frontendDir,
  stdio: 'inherit',
  env: {
    ...process.env,
    NODE_OPTIONS: (`--max-old-space-size=8192 ${process.env.NODE_OPTIONS || ''}`).trim(),
  },
  shell: process.platform === 'win32',
});

if (result.error) {
  console.error(result.error);
  process.exit(result.status ?? 1);
}

process.exit(result.status ?? 0);
