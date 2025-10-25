#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const colors = {
  cyan: '\x1b[0;36m',
  bgBlue: '\x1b[44m',
  reset: '\x1b[0m'
};

process.env.ENV_MODE = 'local';
process.env.MCP_SERVICE_PORT = '3004';

console.log(`${colors.bgBlue}[MCP-SERVICE] Starting on port 3004...${colors.reset}`);

const mcpServicePath = path.join(__dirname, '../../mcp/service');

const service = spawn('node', ['src/server.js'], {
  cwd: mcpServicePath,
  stdio: 'inherit',
  env: { ...process.env }
});

service.on('close', (code) => {
  process.exit(code);
});
