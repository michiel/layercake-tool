#!/usr/bin/env node

const { runFrontend } = require('../src-tauri/scripts/run-frontend.js');

const exitCode = runFrontend(process.argv.slice(2));
process.exit(exitCode);
