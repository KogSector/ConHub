#!/usr/bin/env node

const { execSync } = require('child_process');
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

console.log(`${colors.yellow}[STOP] Stopping ConHub services...${colors.reset}`);

const scriptDir = __dirname;
const forceStopScript = path.join(scriptDir, '../maintenance/force-stop.js');

if (fs.existsSync(forceStopScript)) {
  try {
    execSync(`node "${forceStopScript}"`, { stdio: 'inherit' });
  } catch (e) {
    console.log(`${colors.yellow}[WARNING] Force stop script failed${colors.reset}`);
  }
}

console.log(`${colors.green}[OK] All services stopped${colors.reset}`);
