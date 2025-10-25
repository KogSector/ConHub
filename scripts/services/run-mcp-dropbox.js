#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const colors = {
  cyan: '\x1b[0;36m',
  bgMagenta: '\x1b[45m',
  reset: '\x1b[0m'
};

process.env.ENV_MODE = 'local';

console.log(`${colors.bgMagenta}[MCP-DROPBOX] Starting on port 3007...${colors.reset}`);

const mcpDropboxPath = path.join(__dirname, '../../mcp/servers/dropbox');

const service = spawn('npm', ['start'], {
  cwd: mcpDropboxPath,
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
