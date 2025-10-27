#!/usr/bin/env node

const { spawn } = require('child_process');

// Colors for console output
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m'
};

function log(message, color = colors.reset) {
    console.log(`${color}${message}${colors.reset}`);
}

function logStep(step, message) {
    log(`\n${colors.bright}[${step}]${colors.reset} ${colors.cyan}${message}${colors.reset}`);
}

function logSuccess(message) {
    log(`${colors.green}✓ ${message}${colors.reset}`);
}

function logError(message) {
    log(`${colors.red}✗ ${message}${colors.reset}`);
}

async function runCommand(command, cwd = process.cwd()) {
    return new Promise((resolve, reject) => {
        log(`${colors.blue}Running: ${command}${colors.reset}`);
        
        const child = spawn(command, [], {
            shell: true,
            cwd,
            stdio: 'inherit'
        });

        child.on('close', (code) => {
            if (code === 0) {
                resolve();
            } else {
                reject(new Error(`Command failed with exit code ${code}`));
            }
        });

        child.on('error', (error) => {
            reject(error);
        });
    });
}

async function stopContainers() {
    try {
        logStep('STOP', 'Stopping all ConHub services...');
        await runCommand('docker-compose down');
        logSuccess('All services stopped successfully');
    } catch (error) {
        logError('Failed to stop some containers');
        throw error;
    }
}

async function cleanupContainers() {
    try {
        logStep('CLEANUP', 'Cleaning up containers and networks...');
        await runCommand('docker-compose down --remove-orphans --volumes');
        logSuccess('Cleanup completed');
    } catch (error) {
        logError('Failed to cleanup containers');
        throw error;
    }
}

async function main() {
    const args = process.argv.slice(2);
    const cleanup = args.includes('--cleanup') || args.includes('-c');
    
    try {
        log(`${colors.bright}${colors.magenta}🛑 ConHub Docker Stop Script${colors.reset}\n`);
        
        if (cleanup) {
            await cleanupContainers();
            log(`\n${colors.yellow}⚠ All containers, networks, and volumes have been removed.${colors.reset}`);
            log(`${colors.yellow}You will need to rebuild containers on next start.${colors.reset}`);
        } else {
            await stopContainers();
        }
        
        log(`\n${colors.bright}🏁 ConHub services stopped successfully!${colors.reset}`);
        log(`\n${colors.cyan}To start services again: ${colors.bright}npm start${colors.reset}`);
        
        if (!cleanup) {
            log(`${colors.cyan}To stop and cleanup everything: ${colors.bright}npm run docker:stop -- --cleanup${colors.reset}`);
        }
        
    } catch (error) {
        logError(`Stop failed: ${error.message}`);
        log(`\n${colors.yellow}You can try force stopping with: ${colors.bright}docker-compose down --remove-orphans${colors.reset}`);
        process.exit(1);
    }
}

main();