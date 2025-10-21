#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  magenta: '\x1b[0;35m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';

if (!fs.existsSync('target/debug/lexor')) {
  console.log(`${colors.cyan}[BUILD] Building lexor...${colors.reset}`);
  execSync('cargo build --bin lexor --quiet', { stdio: 'inherit' });
}

console.log(`${colors.magenta}[LEXOR] Starting on port 3002...${colors.reset}`);

const lexor = spawn('./target/debug/lexor', [], { stdio: 'inherit' });

lexor.on('close', (code) => {
  process.exit(code);
});
