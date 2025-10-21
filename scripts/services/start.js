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

const backendBinary = 'target/debug/conhub-backend';
const lexorBinary = 'target/debug/lexor';

if (!fs.existsSync(backendBinary) || !fs.existsSync(lexorBinary)) {
  console.log(`${colors.cyan}[BUILD] Building binaries...${colors.reset}`);
  try {
    execSync('cargo build --bin conhub-backend --bin lexor --quiet', { stdio: 'inherit' });
    console.log(`${colors.green}[OK] Build completed${colors.reset}`);
  } catch (error) {
    console.log(`${colors.red}[ERROR] Build failed${colors.reset}`);
    process.exit(1);
  }
}

console.log(`${colors.cyan}[SERVICES] Starting all services...${colors.reset}`);
console.log('   Frontend: http://localhost:3000');
console.log('   Backend:  http://localhost:3001');
console.log('   Lexor:    http://localhost:3002');
console.log('');

const concurrently = spawn('npx', [
  'concurrently',
  '--names', 'Frontend,Backend,Lexor',
  '--prefix-colors', 'cyan,blue,magenta',
  '--restart-tries', '2',
  '--kill-others-on-fail',
  'npm run dev:frontend',
  'npm run dev:backend',
  'npm run dev:lexor'
], { stdio: 'inherit', shell: true });

concurrently.on('close', (code) => {
  process.exit(code);
});
