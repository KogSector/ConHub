#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

console.log(`${colors.green}[START] Starting ConHub...${colors.reset}`);

// Check if we're in the scripts directory or project root
const scriptsPackageJson = path.join(__dirname, '../package.json');
const rootPackageJson = path.join(__dirname, '../../package.json');

if (!fs.existsSync(scriptsPackageJson) && !fs.existsSync(rootPackageJson)) {
  console.log(`${colors.red}[ERROR] Run from project root or scripts directory${colors.reset}`);
  process.exit(1);
}

console.log(`${colors.yellow}[CLEANUP] Cleaning up ports and locks...${colors.reset}`);
const scriptDir = __dirname;
const cleanupScript = path.join(scriptDir, '../maintenance/cleanup-ports.js');
if (fs.existsSync(cleanupScript)) {
  try {
    execSync(`node "${cleanupScript}"`, { stdio: 'inherit' });
  } catch (e) {
    console.log(`${colors.yellow}[WARNING] Cleanup script not found or failed${colors.reset}`);
  }
}

console.log(`${colors.cyan}[SERVICES] Starting all services...${colors.reset}`);
console.log('   Frontend:         http://localhost:3000');
console.log('   Auth Service:     http://localhost:3010');
console.log('   Billing Service:  http://localhost:3011');
console.log('   AI Service:       http://localhost:3012');
console.log('   Data Service:     http://localhost:3013');
console.log('   Security Service: http://localhost:3014');
console.log('   Webhook Service:  http://localhost:3015');
console.log('   Unified Indexer:  http://localhost:8080');
console.log('   MCP Service:      http://localhost:3004');
console.log('   MCP Google Drive: http://localhost:3005');
console.log('   MCP Filesystem:   http://localhost:3006');
console.log('   MCP Dropbox:      http://localhost:3007');
console.log('');

process.env.ENV_MODE = 'local';

const concurrently = spawn('npm run dev:concurrently', {
  stdio: 'inherit',
  cwd: path.join(__dirname, '..'),
  shell: true
});

// Start docker build in parallel to speed up local dev setup.
// Use the package script which delegates to docker-compose build. We pass -- --parallel
// so the underlying docker-compose receives the --parallel flag.
let dockerBuilder = null;
try {
  dockerBuilder = spawn('npm --prefix .. run docker:build -- --parallel', {
    stdio: 'inherit',
    cwd: path.join(__dirname, '..'),
    shell: true
  });
  dockerBuilder.on('close', (code) => {
    if (code === 0) {
      console.log(`${colors.green}[DOCKER] Parallel build finished successfully${colors.reset}`);
    } else {
      console.log(`${colors.red}[DOCKER] Parallel build exited with code ${code}${colors.reset}`);
    }
  });
  dockerBuilder.on('error', (err) => {
    console.log(`${colors.red}[DOCKER] Failed to start parallel build: ${err.message}${colors.reset}`);
  });
} catch (e) {
  console.log(`${colors.yellow}[DOCKER] Could not start parallel build: ${e.message}${colors.reset}`);
}

let exiting = false;

const projectRoot = path.join(__dirname, '..', '..');

function runCommandSync(cmd, options = {}) {
  try {
    execSync(cmd, { stdio: 'inherit', cwd: options.cwd || projectRoot, timeout: 10 * 60 * 1000 });
    return true;
  } catch (err) {
    console.log(`${colors.red}[ERROR] Command failed: ${cmd}${colors.reset}`);
    return false;
  }
}

const cleanupContainersAndImages = () => {
  if (exiting) return;
  exiting = true;

  console.log(`\n${colors.yellow}[STOP] Cleaning up Docker containers and images...${colors.reset}`);

  // Check for docker availability
  try {
    execSync('docker --version', { stdio: 'ignore' });
  } catch (e) {
    console.log(`${colors.red}[WARNING] Docker not found on PATH; skipping docker cleanup.${colors.reset}`);
    return;
  }

  // Run docker compose down in project root
  const rootCompose = path.join(projectRoot, 'docker-compose.yml');
  if (fs.existsSync(rootCompose)) {
    console.log(`${colors.cyan}[DOCKER] Bringing down root docker-compose...${colors.reset}`);
    runCommandSync('docker compose down --rmi all -v --remove-orphans');
  }

  // If there's a database docker-compose, bring it down too
  const dbCompose = path.join(projectRoot, 'database', 'docker-compose.yml');
  if (fs.existsSync(dbCompose)) {
    console.log(`${colors.cyan}[DOCKER] Bringing down database/docker-compose...${colors.reset}`);
    runCommandSync('docker compose -f database/docker-compose.yml down --rmi all -v --remove-orphans');
  }

  // Optionally run a full prune if explicitly requested (safer opt-in)
  if (process.env.FORCE_DOCKER_PRUNE === 'true') {
    console.log(`${colors.yellow}[DOCKER] FORCE_DOCKER_PRUNE=true — running docker system prune -a -f${colors.reset}`);
    runCommandSync('docker system prune -a -f');
  }

  console.log(`${colors.green}[STOP] Docker cleanup complete.${colors.reset}`);
};

const handleExit = (signal) => {
  console.log(`\n${colors.yellow}[STOP] Received ${signal}, stopping all services...${colors.reset}`);
  try {
    // try to gracefully kill the concurrently process group
    concurrently.kill();
  } catch (e) {
    // ignore
  }
  try {
    if (dockerBuilder && !dockerBuilder.killed) {
      dockerBuilder.kill();
    }
  } catch (e) {
    // ignore
  }

  // Perform docker cleanup synchronously
  try {
    cleanupContainersAndImages();
  } catch (e) {
    console.log(`${colors.red}[ERROR] Cleanup failed: ${e.message}${colors.reset}`);
  }

  // exit after cleanup
  process.exit(0);
};

process.on('SIGINT', () => handleExit('SIGINT'));
process.on('SIGTERM', () => handleExit('SIGTERM'));

concurrently.on('close', (code) => {
  // When concurrently exits, ensure we still perform cleanup so containers/images are removed
  cleanupContainersAndImages();
  process.exit(code);
});
