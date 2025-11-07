#!/usr/bin/env node

/**
 * ConHub Smart Start Script
 * Reads feature-toggles.json and decides whether to use Docker or local builds
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  magenta: '\x1b[1;35m',
  reset: '\x1b[0m',
  bright: '\x1b[1m'
};

function log(message, color = colors.reset) {
  console.log(`${color}${message}${colors.reset}`);
}

function readFeatureToggles() {
  const projectRoot = path.resolve(__dirname, '..');
  const togglesPath = path.join(projectRoot, 'feature-toggles.json');
  
  try {
    if (!fs.existsSync(togglesPath)) {
      log(`⚠ feature-toggles.json not found. Creating default configuration...`, colors.yellow);
      const defaultToggles = {
        Auth: false,
        Heavy: false,
        Docker: false
      };
      fs.writeFileSync(togglesPath, JSON.stringify(defaultToggles, null, 2));
      return defaultToggles;
    }
    
    const content = fs.readFileSync(togglesPath, 'utf8');
    return JSON.parse(content);
  } catch (error) {
    log(`✗ Error reading feature-toggles.json: ${error.message}`, colors.red);
    log(`  Using default configuration (Docker: false)`, colors.yellow);
    return { Auth: false, Heavy: false, Docker: false };
  }
}

function runScript(command, args = []) {
  log(`\n${colors.bright}Executing: ${command} ${args.join(' ')}${colors.reset}\n`);
  
  const child = spawn(command, args, {
    stdio: 'inherit',
    shell: false,
    cwd: __dirname
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

function main() {
  log(`${colors.bright}${colors.magenta}🚀 ConHub Smart Start${colors.reset}\n`);
  
  const toggles = readFeatureToggles();
  const dockerEnabled = toggles.Docker === true;
  
  log(`${colors.cyan}Feature Toggle Status:${colors.reset}`);
  log(`  • Auth:   ${toggles.Auth ? colors.green + 'Enabled' : colors.yellow + 'Disabled'}${colors.reset}`);
  log(`  • Heavy:  ${toggles.Heavy ? colors.green + 'Enabled' : colors.yellow + 'Disabled'}${colors.reset}`);
  log(`  • Docker: ${toggles.Docker ? colors.green + 'Enabled' : colors.yellow + 'Disabled'}${colors.reset}`);
  log('');
  
  if (dockerEnabled) {
    log(`${colors.green}✓ Docker mode enabled${colors.reset}`);
    log(`${colors.cyan}Starting services with Docker...${colors.reset}\n`);
    runScript('node', ['docker/setup-and-run.js']);
  } else {
    log(`${colors.green}✓ Local mode enabled${colors.reset}`);
    log(`${colors.cyan}Starting services locally...${colors.reset}\n`);
    runScript('node', ['services/start.js']);
  }
}

main();
