#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  blue: '\x1b[0;34m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/auth-service')) {
  console.log(`${colors.cyan}[BUILD] Building auth-service...${colors.reset}`);
  execSync('cargo build --bin auth-service', { stdio: 'inherit' });
}

console.log(`${colors.blue}[AUTH] Starting on port 3010...${colors.reset}`);

const service = spawn('./target/debug/auth-service', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
