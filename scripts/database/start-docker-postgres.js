#!/usr/bin/env node

const { spawn, execSync } = require('child_process');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

// Default values
const args = {
  password: 'postgres',
  database: 'conhub',
  port: '5432'
};

// Parse command-line arguments
for (let i = 2; i < process.argv.length; i += 2) {
  const flag = process.argv[i];
  const value = process.argv[i + 1];

  switch (flag) {
    case '-Password':
      args.password = value;
      break;
    case '-Database':
      args.database = value;
      break;
    case '-Port':
      args.port = value;
      break;
    default:
      console.error(`Unknown parameter: ${flag}`);
      process.exit(1);
  }
}

console.log(`${colors.cyan}[DOCKER] Starting PostgreSQL with Docker...${colors.reset}`);

// Check if docker is available
try {
  execSync('docker --version', { stdio: 'ignore' });
} catch (error) {
  console.log(`${colors.red}[ERROR] Docker not found. Please install Docker Desktop.${colors.reset}`);
  process.exit(1);
}

console.log(`${colors.yellow}[CLEANUP] Stopping existing PostgreSQL container...${colors.reset}`);
try {
  execSync('docker stop conhub-postgres', { stdio: 'ignore' });
} catch (e) { /* Container may not exist */ }
try {
  execSync('docker rm conhub-postgres', { stdio: 'ignore' });
} catch (e) { /* Container may not exist */ }

console.log(`${colors.green}[STARTING] Starting PostgreSQL container...${colors.reset}`);

const docker = spawn('docker', [
  'run', '-d',
  '--name', 'conhub-postgres',
  '-e', `POSTGRES_PASSWORD=${args.password}`,
  '-e', `POSTGRES_DB=${args.database}`,
  '-p', `${args.port}:5432`,
  'postgres:15'
], { stdio: 'inherit' });

docker.on('close', (code) => {
  if (code === 0) {
    console.log(`${colors.green}[SUCCESS] PostgreSQL started successfully!${colors.reset}`);
    console.log(`${colors.cyan}Connection details:${colors.reset}`);
    console.log(`  Host: localhost`);
    console.log(`  Port: ${args.port}`);
    console.log(`  Database: ${args.database}`);
    console.log(`  Username: postgres`);
    console.log(`  Password: ${args.password}`);
    console.log('');
    console.log(`${colors.green}Now you can run:${colors.reset}`);
    console.log(`./clear-database.sh -Password '${args.password}'`);
    console.log('');
    console.log(`${colors.yellow}To stop the container later:${colors.reset}`);
    console.log('docker stop conhub-postgres');
  } else {
    console.log(`${colors.red}[ERROR] Failed to start PostgreSQL container${colors.reset}`);
  }
  process.exit(code);
});
