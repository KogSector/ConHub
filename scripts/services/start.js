#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

console.log(`${colors.green}[START] Starting ConHub...${colors.reset}`);

// Check if we're in the scripts directory or project root
const scriptsPackageJson = path.join(__dirname, '../package.json');
const rootPackageJson = path.join(__dirname, '../../package.json');

if (!fs.existsSync(scriptsPackageJson) && !fs.existsSync(rootPackageJson)) {
  console.log(`${colors.red}[ERROR] Run from project root or scripts directory${colors.reset}`);
  process.exit(1);
}

console.log(`${colors.yellow}[CLEANUP] Cleaning up ports and locks...${colors.reset}`);
const scriptDir = __dirname;
const cleanupScript = path.join(scriptDir, '../maintenance/cleanup-ports.js');
if (fs.existsSync(cleanupScript)) {
  try {
    execSync(`node "${cleanupScript}"`, { stdio: 'inherit' });
  } catch (e) {
    console.log(`${colors.yellow}[WARNING] Cleanup script not found or failed${colors.reset}`);
  }
}

console.log(`${colors.cyan}[SERVICES] Starting all services...${colors.reset}`);
console.log('   Frontend:         http://localhost:3000');
console.log('   Auth Service:     http://localhost:3010');
console.log('   Billing Service:  http://localhost:3011');
console.log('   AI Service:       http://localhost:3012');
console.log('   Data Service:     http://localhost:3013');
console.log('   Security Service: http://localhost:3014');
console.log('   Webhook Service:  http://localhost:3015');
console.log('   Unified Indexer:  http://localhost:8080');
console.log('   MCP Service:      http://localhost:3004');
console.log('   MCP Google Drive: http://localhost:3005');
console.log('   MCP Filesystem:   http://localhost:3006');
console.log('   MCP Dropbox:      http://localhost:3007');
console.log('');

process.env.ENV_MODE = 'local';

const concurrently = spawn('npm run dev:concurrently', {
  stdio: 'inherit',
  cwd: path.join(__dirname, '..'),
  shell: true
});

// Note: Docker builds are now controlled by feature-toggles.json (Docker key)
// Use "npm start" with Docker: true to enable Docker builds
// Use "npm start" with Docker: false for local development only

// Note: Docker-related functions removed as Docker mode is now handled separately
// via the Docker toggle feature. Use "npm run docker:stop" to stop Docker containers.

const handleExit = (signal) => {
  console.log(`\n${colors.yellow}[STOP] Received ${signal}, stopping all services...${colors.reset}`);
  try {
    // try to gracefully kill the concurrently process group
    concurrently.kill();
  } catch (e) {
    // ignore
  }

  // Note: Docker cleanup removed - only needed when Docker mode is enabled
  // Docker containers are managed separately via docker:stop command

  // exit after cleanup
  process.exit(0);
};

process.on('SIGINT', () => handleExit('SIGINT'));
process.on('SIGTERM', () => handleExit('SIGTERM'));

concurrently.on('close', (code) => {
  // Local mode - no Docker cleanup needed
  process.exit(code);
});
