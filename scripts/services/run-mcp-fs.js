#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const colors = {
  cyan: '\x1b[0;36m',
  bgYellow: '\x1b[43m',
  reset: '\x1b[0m'
};

process.env.ENV_MODE = 'local';

console.log(`${colors.bgYellow}[MCP-FS] Starting on port 3006...${colors.reset}`);

const mcpFsPath = path.join(__dirname, '../../mcp/servers/filesystem');

const service = spawn('npm', ['start'], {
  cwd: mcpFsPath,
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
