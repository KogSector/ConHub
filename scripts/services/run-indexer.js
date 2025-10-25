#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  white: '\x1b[0;97m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

if (!fs.existsSync('target/debug/unified-indexer')) {
  console.log(`${colors.cyan}[BUILD] Building unified-indexer...${colors.reset}`);
  execSync('cargo build --bin unified-indexer', { stdio: 'inherit' });
}

console.log(`${colors.white}[INDEXER] Starting on port 8080...${colors.reset}`);

const service = spawn('./target/debug/unified-indexer', [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
