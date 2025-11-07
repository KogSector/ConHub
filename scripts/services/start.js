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
console.log('   MCP Service:      http://localhost:3004');
console.log('   MCP Google Drive: http://localhost:3005');
console.log('   MCP Filesystem:   http://localhost:3006');
console.log('   MCP Dropbox:      http://localhost:3007');
console.log('');

process.env.ENV_MODE = 'local';

// Resolve concurrently binary explicitly to avoid PATH issues on Windows
const projectRoot = path.join(__dirname, '..');
const isWin = process.platform === 'win32';
const concurrentlyBin = path.join(projectRoot, 'node_modules', '.bin', isWin ? 'concurrently.cmd' : 'concurrently');

let concurrentlyArgs = [
  '--names', 'Frontend,Auth,Billing,Client,Data,Security,Webhook,MCP-Svc,MCP-GDrive,MCP-FS,MCP-Dropbox',
  '--prefix-colors', 'cyan,blue,magenta,green,yellow,red,gray,bgBlue,bgGreen,bgYellow,bgMagenta',
  '--restart-tries', '2',
  '--kill-others-on-fail',
  'npm --prefix .. run dev:frontend',
  'npm --prefix .. run dev:auth',
  'npm --prefix .. run dev:billing',
  'npm --prefix .. run dev:client',
  'npm --prefix .. run dev:data',
  'npm --prefix .. run dev:security',
  'npm --prefix .. run dev:webhook',
  'npm --prefix .. run dev:mcp-service',
  "echo 'MCP GDrive not implemented'",
  "echo 'MCP FS not implemented'",
  "echo 'MCP Dropbox not implemented'"
];

// Prefer invoking via npm which sets PATH for node_modules/.bin reliably
const npmCmd = isWin ? 'npm.cmd' : 'npm';
const child = spawn(npmCmd, ['run', 'dev:concurrently'], {
  stdio: 'inherit',
  cwd: projectRoot,
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
    child.kill();
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

child.on('close', (code) => {
  // Local mode - no Docker cleanup needed
  process.exit(code);
});
