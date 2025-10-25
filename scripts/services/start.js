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

if (!fs.existsSync('package.json')) {
  console.log(`${colors.red}[ERROR] Run from project root${colors.reset}`);
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

const concurrently = spawn('npx', [
  'concurrently',
  '--names', 'Frontend,Auth,Billing,AI,Data,Security,Webhook,Indexer,MCP-Svc,MCP-GDrive,MCP-FS,MCP-Dropbox',
  '--prefix-colors', 'cyan,blue,magenta,green,yellow,red,gray,white,bgBlue,bgGreen,bgYellow,bgMagenta',
  '--restart-tries', '2',
  '--kill-others-on-fail',
  'npm run dev:frontend',
  'npm run dev:auth',
  'npm run dev:billing',
  'npm run dev:ai',
  'npm run dev:data',
  'npm run dev:security',
  'npm run dev:webhook',
  'npm run dev:indexer',
  'npm run dev:mcp-service',
  'npm run dev:mcp-gdrive',
  'npm run dev:mcp-fs',
  'npm run dev:mcp-dropbox'
], { stdio: 'inherit', shell: true });

concurrently.on('close', (code) => {
  process.exit(code);
});
