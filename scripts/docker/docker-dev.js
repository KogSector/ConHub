#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const fs = require('fs');
const http = require('http');

// ANSI color codes
const colors = {
  green: '\x1b[0;32m',
  yellow: '\x1b[1;33m',
  red: '\x1b[0;31m',
  blue: '\x1b[0;34m',
  reset: '\x1b[0m'
};

// Parse arguments
let action = 'up';
let service = '';
let build = false;

for (let i = 2; i < process.argv.length; i++) {
  const arg = process.argv[i];
  if (['up', 'down', 'restart', 'logs', 'build', 'clean', 'help', 'h'].includes(arg)) {
    action = arg;
  } else if (arg === '-Service') {
    service = process.argv[++i];
  } else if (arg === '-Build') {
    build = true;
  }
}

function log(msg, color = colors.reset) {
  console.log(`${color}${msg}${colors.reset}`);
}

function showHeader() {
  log('🐳 ConHub Docker Development Environment', colors.blue);
  log('=======================================', colors.blue);
  console.log('');
}

function checkPrerequisites() {
  log('🔍 Checking prerequisites...', colors.yellow);

  try {
    const dockerVersion = execSync('docker --version', { encoding: 'utf8' }).trim();
    log(`✅ Docker found: ${dockerVersion}`, colors.green);
  } catch (error) {
    log('❌ Docker is not installed or not running', colors.red);
    log('Please install Docker Desktop and ensure it\'s running', colors.red);
    process.exit(1);
  }

  try {
    const composeVersion = execSync('docker-compose --version', { encoding: 'utf8' }).trim();
    log(`✅ Docker Compose found: ${composeVersion}`, colors.green);
  } catch (error) {
    log('❌ Docker Compose is not available', colors.red);
    process.exit(1);
  }

  if (fs.existsSync('.env')) {
    log('✅ Environment file found', colors.green);
  } else {
    log('⚠️  .env file not found, using .env.example', colors.yellow);
    if (fs.existsSync('.env.example')) {
      fs.copyFileSync('.env.example', '.env');
      log('✅ Created .env from .env.example', colors.green);
    } else {
      log('❌ .env.example not found', colors.red);
      process.exit(1);
    }
  }
  console.log('');
}

function startServices() {
  log('🚀 Starting ConHub services...', colors.yellow);

  const args = ['up', '-d'];
  if (build) args.push('--build');
  if (service) {
    args.push(service);
    log(`Starting service: ${service}`, colors.blue);
  } else {
    log('Starting all services...', colors.blue);
  }

  const compose = spawn('docker-compose', args, { stdio: 'inherit' });
  compose.on('close', (code) => {
    if (code === 0) {
      log('✅ Services started successfully!', colors.green);
    } else {
      log('❌ Failed to start services', colors.red);
      process.exit(1);
    }
  });
}

function stopServices() {
  log('🛑 Stopping ConHub services...', colors.yellow);
  const args = service ? ['stop', service] : ['down'];
  execSync(`docker-compose ${args.join(' ')}`, { stdio: 'inherit' });
  log(service ? `✅ Service ${service} stopped` : '✅ All services stopped', colors.green);
}

function restartServices() {
  log('🔄 Restarting ConHub services...', colors.yellow);
  stopServices();
  setTimeout(() => startServices(), 2000);
}

function showLogs() {
  log('📋 Showing service logs...', colors.yellow);
  const args = service ? ['logs', '-f', service] : ['logs', '-f'];
  spawn('docker-compose', args, { stdio: 'inherit' });
}

function buildServices() {
  log('🔨 Building ConHub services...', colors.yellow);
  const args = service ? ['build', service] : ['build'];
  execSync(`docker-compose ${args.join(' ')}`, { stdio: 'inherit' });
  log(service ? `✅ Service ${service} built` : '✅ All services built', colors.green);
}

function cleanEnvironment() {
  log('🧹 Cleaning Docker environment...', colors.yellow);
  execSync('docker-compose down -v --remove-orphans', { stdio: 'inherit' });
  execSync('docker image prune -f', { stdio: 'inherit' });
  execSync('docker volume prune -f', { stdio: 'inherit' });
  execSync('docker network prune -f', { stdio: 'inherit' });
  log('✅ Environment cleaned successfully', colors.green);
}

function showHelp() {
  log('ConHub Docker Development Commands:', colors.blue);
  console.log('');
  console.log('  up       - Start all services (default)');
  console.log('  down     - Stop all services');
  console.log('  restart  - Restart all services');
  console.log('  logs     - Show service logs');
  console.log('  build    - Build services');
  console.log('  clean    - Clean Docker environment');
  console.log('');
  console.log('Options:');
  console.log('  -Service <name>  - Target specific service');
  console.log('  -Build           - Build images before starting');
  console.log('');
}

showHeader();

if (action === 'help' || action === 'h') {
  showHelp();
  process.exit(0);
}

checkPrerequisites();

switch (action) {
  case 'up': startServices(); break;
  case 'down': stopServices(); break;
  case 'restart': restartServices(); break;
  case 'logs': showLogs(); break;
  case 'build': buildServices(); break;
  case 'clean': cleanEnvironment(); break;
  default:
    log(`❌ Unknown action: ${action}`, colors.red);
    showHelp();
    process.exit(1);
}
