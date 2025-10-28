#!/usr/bin/env node

/**
 * ConHub Orchestration Script
 * Simple launcher that delegates to the scripts microservice
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

function showUsage() {
  console.log(`${colors.cyan}ConHub Orchestration Commands:${colors.reset}`);
  console.log('');
  console.log('  node start.js dev              - Start all services in development mode');
  console.log('  node start.js dev:concurrently - Start all services with concurrently');
  console.log('  node start.js docker           - Start all services with Docker');
  console.log('  node start.js stop             - Stop all services');
  console.log('  node start.js status           - Check service status');
  console.log('  node start.js frontend         - Start only frontend');
  console.log('  node start.js cleanup          - Clean up ports and processes');
  console.log('');
  console.log(`${colors.yellow}Note: This script delegates to the scripts microservice${colors.reset}`);
}

function runScript(scriptPath, args = []) {
  const fullPath = path.join(__dirname, 'scripts', scriptPath);
  
  if (!fs.existsSync(fullPath)) {
    console.log(`${colors.red}[ERROR] Script not found: ${fullPath}${colors.reset}`);
    process.exit(1);
  }

  const child = spawn('node', [fullPath, ...args], {
    stdio: 'inherit',
    cwd: path.join(__dirname, 'scripts')
  });

  child.on('close', (code) => {
    process.exit(code);
  });

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    child.kill('SIGINT');
  });
  
  process.on('SIGTERM', () => {
    child.kill('SIGTERM');
  });
}

const command = process.argv[2];

switch (command) {
  case 'dev':
  case 'development':
    console.log(`${colors.green}[CONHUB] Starting development services...${colors.reset}`);
    runScript('services/start.js');
    break;
    
  case 'dev:concurrently':
      console.log(`${colors.green}[CONHUB] Starting all services with concurrently...${colors.reset}`);
      runCommand('npm run dev:concurrently');
      break;
    
  case 'docker':
    console.log(`${colors.green}[CONHUB] Starting Docker services...${colors.reset}`);
    runScript('docker/setup-and-run.js');
    break;
    
  case 'stop':
    console.log(`${colors.yellow}[CONHUB] Stopping services...${colors.reset}`);
    runScript('services/stop.js');
    break;
    
  case 'status':
    console.log(`${colors.cyan}[CONHUB] Checking service status...${colors.reset}`);
    runScript('services/status.js');
    break;
    
  case 'frontend':
    console.log(`${colors.cyan}[CONHUB] Starting frontend only...${colors.reset}`);
    const frontendChild = spawn('npm', ['run', 'dev'], {
      stdio: 'inherit',
      cwd: path.join(__dirname, 'frontend')
    });
    
    frontendChild.on('close', (code) => {
      process.exit(code);
    });
    break;
    
  case 'cleanup':
    console.log(`${colors.yellow}[CONHUB] Cleaning up...${colors.reset}`);
    runScript('maintenance/cleanup-ports.js');
    break;
    
  case 'help':
  case '--help':
  case '-h':
    showUsage();
    break;
    
  default:
    if (!command) {
      console.log(`${colors.red}[ERROR] No command specified${colors.reset}`);
    } else {
      console.log(`${colors.red}[ERROR] Unknown command: ${command}${colors.reset}`);
    }
    console.log('');
    showUsage();
    process.exit(1);
}