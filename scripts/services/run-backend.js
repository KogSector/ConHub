#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  blue: '\x1b[0;34m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';

if (!fs.existsSync('target/debug/conhub-backend')) {
  console.log(`${colors.cyan}[BUILD] Building backend...${colors.reset}`);
  execSync('cargo build --bin conhub-backend --quiet', { stdio: 'inherit' });
}

console.log(`${colors.blue}[BACKEND] Starting on port 3001...${colors.reset}`);

const backend = spawn('./target/debug/conhub-backend', [], { stdio: 'inherit' });

backend.on('close', (code) => {
  process.exit(code);
});
