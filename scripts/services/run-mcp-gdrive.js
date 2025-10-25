#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const colors = {
  cyan: '\x1b[0;36m',
  bgGreen: '\x1b[42m',
  reset: '\x1b[0m'
};

process.env.ENV_MODE = 'local';

console.log(`${colors.bgGreen}[MCP-GDRIVE] Starting on port 3005...${colors.reset}`);

const mcpGdrivePath = path.join(__dirname, '../../mcp/servers/google-drive');

const service = spawn('npm', ['start'], {
  cwd: mcpGdrivePath,
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
