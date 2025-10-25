#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  yellow: '\x1b[0;33m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/data-service')) {
  console.log(`${colors.cyan}[BUILD] Building data-service...${colors.reset}`);
  execSync('cargo build --bin data-service', { stdio: 'inherit' });
}

console.log(`${colors.yellow}[DATA] Starting on port 3013...${colors.reset}`);

const service = spawn('./target/debug/data-service', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
