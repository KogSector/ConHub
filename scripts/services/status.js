#!/usr/bin/env node

const http = require('http');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  reset: '\x1b[0m'
};

const SERVICES = [
  { name: 'Frontend', port: 3000 },
  { name: 'Backend', port: 3001 },
  { name: 'Lexor', port: 3002 },
  { name: 'Doc Search', port: 8001 },
  { name: 'Langchain Service', port: 8003 }
];

console.log(`${colors.cyan}[STATUS] Checking ConHub services...${colors.reset}`);

function checkService(name, port) {
  return new Promise((resolve) => {
    const req = http.get(`http://localhost:${port}`, { timeout: 2000 }, (res) => {
      resolve(true);
      req.abort();
    });

    req.on('error', () => {
      resolve(false);
    });

    req.on('timeout', () => {
      resolve(false);
      req.abort();
    });
  });
}

async function checkAllServices() {
  for (const service of SERVICES) {
    const isRunning = await checkService(service.name, service.port);

    if (isRunning) {
      console.log(`${colors.green}✓ ${service.name} - Running on port ${service.port}${colors.reset}`);
    } else {
      console.log(`${colors.red}✗ ${service.name} - Not responding on port ${service.port}${colors.reset}`);
    }
  }
}

checkAllServices();
