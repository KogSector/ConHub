#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  magenta: '\x1b[0;35m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/billing-service')) {
  console.log(`${colors.cyan}[BUILD] Building billing-service...${colors.reset}`);
  execSync('cargo build --bin billing-service', { stdio: 'inherit' });
}

console.log(`${colors.magenta}[BILLING] Starting on port 3011...${colors.reset}`);

const service = spawn('./target/debug/billing-service', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
