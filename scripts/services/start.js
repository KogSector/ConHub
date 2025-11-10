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

function readFeatureToggles() {
  const togglesPath = path.resolve(__dirname, '..', 'feature-toggles.json');
  try {
    if (!fs.existsSync(togglesPath)) {
      return { Auth: false, Heavy: false, Docker: false };
    }
    const content = fs.readFileSync(togglesPath, 'utf8');
    return JSON.parse(content);
  } catch (_) {
    return { Auth: false, Heavy: false, Docker: false };
  }
}

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

const toggles = readFeatureToggles();
const authEnabled = toggles.Auth === true;

console.log(`${colors.cyan}[SERVICES] Starting services (Auth: ${authEnabled ? 'enabled' : 'disabled'})...${colors.reset}`);
console.log('   Frontend:         http://localhost:3000');
if (authEnabled) console.log('   Auth Service:     http://localhost:3010');
// Core services always available regardless of Heavy toggle
console.log('   Billing Service:  http://localhost:3011');
console.log('   AI Service:       http://localhost:3012');
console.log('   Data Service:     http://localhost:3013');
console.log('   Security Service: http://localhost:3014');
console.log('   Webhook Service:  http://localhost:3015');
// Heavy-only services: embeddings and indexers
if (toggles.Heavy === true) {
  console.log('   Indexer Service:  http://localhost:8080');
  console.log('   Embedding Service:http://localhost:8082');
} else {
  console.log('   Indexer Service:  disabled (Heavy=false)');
  console.log('   Embedding Service:disabled (Heavy=false)');
}
console.log('');

process.env.ENV_MODE = 'local';

// Use concurrently programmatic API to avoid CLI arg parsing quirks
const projectRoot = path.join(__dirname, '..');
const isWin = process.platform === 'win32';
const concurrentlyDefault = require('concurrently').default || require('concurrently');

const heavyEnabled = toggles.Heavy === true;

const names = ['Frontend'];
const prefixColors = ['cyan'];
const commands = ['npm --prefix .. run dev:frontend'];

// Auth follows its own toggle, independent of Heavy
if (authEnabled) {
  names.push('Auth');
  prefixColors.push('blue');
  commands.push('npm --prefix .. run dev:auth');
}

// Core services should run regardless of Heavy
names.push('Billing','Client','Data','Security','Webhook');
prefixColors.push('magenta','green','yellow','red','gray');
commands.push(
  'npm --prefix .. run dev:billing',
  'npm --prefix .. run dev:client',
  'npm --prefix .. run dev:data',
  'npm --prefix .. run dev:security',
  'npm --prefix .. run dev:webhook'
);

// Heavy-only services: start when Heavy=true
if (heavyEnabled) {
  names.push('Indexer','Embedding');
  prefixColors.push('white','white');
  commands.push(
    'npm --prefix .. run dev:indexer',
    'npm --prefix .. run dev:embedding'
  );
}

const commandObjs = commands.map((cmd, idx) => ({ command: cmd, name: names[idx] }));
const concurrentlyOpts = {
  prefix: 'name',
  prefixColors,
  restartTries: 2,
  killOthersOn: ['failure'],
  raw: false,
  cwd: projectRoot,
};

// Prefer invoking via npm which sets PATH for node_modules/.bin reliably
// Run via library to avoid yargs converting --prefix to boolean
concurrentlyDefault(commandObjs, concurrentlyOpts).result.then(
  () => process.exit(0),
  () => process.exit(1)
);

// Note: Docker builds are now controlled by feature-toggles.json (Docker key)
// Use "npm start" with Docker: true to enable Docker builds
// Use "npm start" with Docker: false for local development only

// Note: Docker-related functions removed as Docker mode is now handled separately
// via the Docker toggle feature. Use "npm run docker:stop" to stop Docker containers.

process.on('SIGINT', () => {
  console.log(`\n${colors.yellow}[STOP] Received SIGINT, stopping all services...${colors.reset}`);
  process.exit(0);
});
process.on('SIGTERM', () => {
  console.log(`\n${colors.yellow}[STOP] Received SIGTERM, stopping all services...${colors.reset}`);
  process.exit(0);
});
