#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  gray: '\x1b[0;37m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/webhook-service')) {
  console.log(`${colors.cyan}[BUILD] Building webhook-service...${colors.reset}`);
  execSync('cargo build --bin webhook-service', { stdio: 'inherit' });
}

console.log(`${colors.gray}[WEBHOOK] Starting on port 3015...${colors.reset}`);

const service = spawn('./target/debug/webhook-service', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
