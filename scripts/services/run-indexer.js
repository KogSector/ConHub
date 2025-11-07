#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const colors = {
  cyan: '\x1b[0;36m',
  white: '\x1b[0;97m',
  reset: '\x1b[0m'
};

process.env.RUST_LOG = 'info';
process.env.RUST_BACKTRACE = '1';
process.env.ENV_MODE = 'local';

const serviceDir = path.join(process.cwd(), 'indexers');
const exeSuffix = process.platform === 'win32' ? '.exe' : '';
const binPath = path.join(serviceDir, 'target', 'debug', `unified-indexer${exeSuffix}`);

if (!fs.existsSync(binPath)) {
  console.log(`${colors.cyan}[BUILD] Building unified-indexer...${colors.reset}`);
  execSync('cargo build --bin unified-indexer', { stdio: 'inherit', cwd: serviceDir });
}

console.log(`${colors.white}[INDEXER] Starting on port 8080...${colors.reset}`);

const service = spawn(binPath, [], {
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
