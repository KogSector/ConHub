#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/ai-service')) {
  console.log(`${colors.cyan}[BUILD] Building ai-service...${colors.reset}`);
  execSync('cargo build --bin ai-service', { stdio: 'inherit' });
}

console.log(`${colors.green}[AI] Starting on port 3012...${colors.reset}`);

const service = spawn('./target/debug/ai-service', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
