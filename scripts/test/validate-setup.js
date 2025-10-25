#!/usr/bin/env node

const http = require('http');
const { execSync } = require('child_process');

const colors = {
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  cyan: '\x1b[0;36m',
  reset: '\x1b[0m'
};

const LOCAL_SERVICES = [
  { name: 'Frontend', url: 'http://localhost:3000', path: '/' },
  { name: 'Auth', url: 'http://localhost:3010', path: '/health' },
  { name: 'Billing', url: 'http://localhost:3011', path: '/health' },
  { name: 'AI', url: 'http://localhost:3012', path: '/health' },
  { name: 'Data', url: 'http://localhost:3013', path: '/health' },
  { name: 'Security', url: 'http://localhost:3014', path: '/health' },
  { name: 'Webhook', url: 'http://localhost:3015', path: '/health' },
  { name: 'Indexer', url: 'http://localhost:8080', path: '/health' },
  { name: 'MCP Service', url: 'http://localhost:3004', path: '/api/health' }
];

function checkService(service) {
  return new Promise((resolve) => {
    const url = new URL(service.url + service.path);
    http.get(url, (res) => {
      if (res.statusCode === 200 || res.statusCode === 404) {
        console.log(`${colors.green}✓${colors.reset} ${service.name} is running`);
        resolve(true);
      } else {
        console.log(`${colors.yellow}⚠${colors.reset} ${service.name} returned ${res.statusCode}`);
        resolve(false);
      }
    }).on('error', (err) => {
      console.log(`${colors.red}✗${colors.reset} ${service.name} is not accessible`);
      resolve(false);
    });
  });
}

async function validateLocal() {
  console.log(`\n${colors.cyan}=== Validating Local Setup ===${colors.reset}\n`);

  // Check databases
  console.log('Checking databases...');
  try {
    execSync('docker ps | grep conhub-postgres', { stdio: 'pipe' });
    console.log(`${colors.green}✓${colors.reset} PostgreSQL is running`);
  } catch {
    console.log(`${colors.red}✗${colors.reset} PostgreSQL is not running`);
    return false;
  }

  // Check services
  console.log('\nChecking services...');
  const results = await Promise.all(LOCAL_SERVICES.map(checkService));
  const allPassed = results.every(r => r);

  if (allPassed) {
    console.log(`\n${colors.green}✓ All local services are running correctly${colors.reset}\n`);
  } else {
    console.log(`\n${colors.red}✗ Some services are not running${colors.reset}\n`);
  }

  return allPassed;
}

async function validateDocker() {
  console.log(`\n${colors.cyan}=== Validating Docker Setup ===${colors.reset}\n`);

  // Check containers
  console.log('Checking Docker containers...');
  try {
    const output = execSync('docker ps --filter "name=conhub" --format "{{.Names}}"', { encoding: 'utf-8' });
    const containers = output.trim().split('\n').filter(Boolean);
    console.log(`${colors.green}✓${colors.reset} Found ${containers.length} ConHub containers`);

    if (containers.length < 16) {
      console.log(`${colors.yellow}⚠${colors.reset} Expected 16 containers, found ${containers.length}`);
    }

    containers.forEach(name => {
      console.log(`  - ${name}`);
    });
  } catch {
    console.log(`${colors.red}✗${colors.reset} No ConHub containers found`);
    return false;
  }

  console.log(`\n${colors.green}✓ Docker setup validated${colors.reset}\n`);
  return true;
}

async function main() {
  const mode = process.argv[2] || 'local';

  if (mode === 'local') {
    await validateLocal();
  } else if (mode === 'docker') {
    await validateDocker();
  } else {
    console.log('Usage: node validate-setup.js [local|docker]');
  }
}

main();
