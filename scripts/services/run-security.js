#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  red: '\x1b[0;31m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/security-service')) {
  console.log(`${colors.cyan}[BUILD] Building security-service...${colors.reset}`);
  execSync('cargo build --bin security-service', { stdio: 'inherit' });
}

console.log(`${colors.red}[SECURITY] Starting on port 3014...${colors.reset}`);

const service = spawn('./target/debug/security-service', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
