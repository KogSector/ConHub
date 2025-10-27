#!/usr/bin/env node

const { spawn, exec } = require('child_process');

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

function logWarning(message) {
    log(`${colors.yellow}⚠ ${message}${colors.reset}`);
}

function logInfo(message) {
    log(`${colors.blue}ℹ ${message}${colors.reset}`);
}

async function runCommand(command, cwd = process.cwd(), silent = false) {
    return new Promise((resolve, reject) => {
        if (!silent) {
            log(`${colors.blue}Running: ${command}${colors.reset}`);
        }
        
        const child = spawn(command, [], {
            shell: true,
            cwd,
            stdio: silent ? 'pipe' : 'inherit'
        });

        let output = '';
        if (silent) {
            child.stdout.on('data', (data) => {
                output += data.toString();
            });
            child.stderr.on('data', (data) => {
                output += data.toString();
            });
        }

        child.on('close', (code) => {
            if (code === 0) {
                resolve(output);
            } else {
                reject(new Error(`Command failed with exit code ${code}: ${output}`));
            }
        });

        child.on('error', (error) => {
            reject(error);
        });
    });
}

async function checkDockerRunning() {
    try {
        await runCommand('docker info', process.cwd(), true);
        return true;
    } catch (error) {
        return false;
    }
}

async function getRunningContainers() {
    try {
        const output = await runCommand('docker-compose ps --services --filter "status=running"', process.cwd(), true);
        return output.split('\n').filter(line => line.trim() && !line.includes('time=') && !line.includes('level=warning'));
    } catch (error) {
        return [];
    }
}

async function stopContainers() {
    try {
        logStep('STOP', 'Stopping all ConHub services...');
        await runCommand('docker-compose down');
        logSuccess('All services stopped successfully');
        return true;
    } catch (error) {
        logError('Failed to stop some containers');
        return false;
    }
}

async function forceStopContainers() {
    try {
        logStep('FORCE-STOP', 'Force stopping all ConHub containers...');
        await runCommand('docker-compose down --remove-orphans');
        logSuccess('All containers force stopped');
        return true;
    } catch (error) {
        logError('Failed to force stop containers');
        return false;
    }
}

async function cleanupContainers() {
    try {
        logStep('CLEANUP', 'Stopping and removing containers, networks, and volumes...');
        await runCommand('docker-compose down --remove-orphans --volumes');
        logSuccess('Complete cleanup completed');
        return true;
    } catch (error) {
        logError('Failed to cleanup containers');
        return false;
    }
}

async function showContainerStatus() {
    try {
        logStep('STATUS', 'Checking remaining containers...');
        const runningContainers = await getRunningContainers();
        
        if (runningContainers.length === 0) {
            logSuccess('No ConHub containers are running');
        } else {
            logWarning(`${runningContainers.length} containers may still be running:`);
            runningContainers.forEach(container => {
                log(`  • ${container}`);
            });
        }
    } catch (error) {
        logWarning('Could not check container status');
    }
}

async function main() {
    const args = process.argv.slice(2);
    const forceStop = args.includes('--force') || args.includes('-f');
    const cleanup = args.includes('--cleanup') || args.includes('-c');
    const helpRequested = args.includes('--help') || args.includes('-h');
    
    if (helpRequested) {
        log(`${colors.bright}${colors.magenta}🛑 ConHub Docker Stop Script${colors.reset}\n`);
        log(`${colors.cyan}Description:${colors.reset}`);
        log(`  Stops all ConHub Docker containers without deleting them.`);
        log(`  Containers can be restarted quickly with 'npm run docker:setup'\n`);
        
        log(`${colors.cyan}Usage:${colors.reset}`);
        log(`  npm run docker:stop                    # Stop all containers (preserves images)`);
        log(`  npm run docker:stop -- --force        # Force stop containers and remove orphans`);
        log(`  npm run docker:stop -- --cleanup      # Stop and remove containers, networks, volumes`);
        log(`  npm run docker:stop -- --help         # Show this help\n`);
        
        log(`${colors.cyan}Flags:${colors.reset}`);
        log(`  --force, -f      Force stop and remove orphaned containers`);
        log(`  --cleanup, -c    Complete cleanup (removes containers, networks, volumes)`);
        log(`  --help, -h       Show this help message\n`);
        
        log(`${colors.yellow}Note:${colors.reset}`);
        log(`  • Default stop preserves containers for quick restart`);
        log(`  • Use --cleanup only if you want to completely reset the environment`);
        log(`  • Docker images are preserved unless you run --cleanup`);
        
        return;
    }
    
    try {
        log(`${colors.bright}${colors.magenta}🛑 ConHub Docker Stop Script${colors.reset}\n`);
        
        // Check if Docker is running
        logStep('DOCKER', 'Checking Docker status...');
        const dockerRunning = await checkDockerRunning();
        if (!dockerRunning) {
            logWarning('Docker is not running. Containers may already be stopped.');
            logInfo('If Docker Desktop is not running, containers are automatically stopped.');
            return;
        }
        logSuccess('Docker is running');

        // Check what containers are currently running
        const runningContainers = await getRunningContainers();
        if (runningContainers.length === 0) {
            logInfo('No ConHub containers are currently running');
            log(`\n${colors.bright}✅ All ConHub services are already stopped!${colors.reset}`);
            return;
        }
        
        logInfo(`Found ${runningContainers.length} running ConHub services`);
        
        // Stop containers based on the mode
        let success = false;
        
        if (cleanup) {
            success = await cleanupContainers();
            if (success) {
                log(`\n${colors.yellow}⚠ Complete cleanup performed:${colors.reset}`);
                log(`  • All containers removed`);
                log(`  • All networks removed`);
                log(`  • All volumes removed`);
                log(`  • Docker images preserved`);
                log(`\n${colors.cyan}To restart ConHub: ${colors.bright}npm run docker:setup${colors.reset}`);
                log(`${colors.yellow}Note: Next startup will rebuild containers${colors.reset}`);
            }
        } else if (forceStop) {
            success = await forceStopContainers();
            if (success) {
                log(`\n${colors.yellow}⚠ Force stop completed:${colors.reset}`);
                log(`  • All containers stopped`);
                log(`  • Orphaned containers removed`);
                log(`  • Docker images preserved`);
                log(`\n${colors.cyan}To restart ConHub: ${colors.bright}npm run docker:setup${colors.reset}`);
            }
        } else {
            success = await stopContainers();
            if (success) {
                log(`\n${colors.green}✅ Clean stop completed:${colors.reset}`);
                log(`  • All containers stopped gracefully`);
                log(`  • Containers preserved for quick restart`);
                log(`  • Docker images preserved`);
                log(`\n${colors.cyan}To restart ConHub: ${colors.bright}npm run docker:setup${colors.reset}`);
            }
        }
        
        if (!success) {
            logError('Stop operation failed');
            log(`\n${colors.yellow}Troubleshooting options:${colors.reset}`);
            log(`  • Try force stop: ${colors.bright}npm run docker:stop -- --force${colors.reset}`);
            log(`  • Complete cleanup: ${colors.bright}npm run docker:stop -- --cleanup${colors.reset}`);
            log(`  • Manual stop: ${colors.bright}docker-compose down --remove-orphans${colors.reset}`);
            process.exit(1);
        }
        
        // Show final status
        await showContainerStatus();
        
        log(`\n${colors.bright}🏁 ConHub Docker stop completed!${colors.reset}`);
        
    } catch (error) {
        logError(`Stop failed: ${error.message}`);
        log(`\n${colors.yellow}Emergency stop options:${colors.reset}`);
        log(`  • Force stop all Docker containers: ${colors.bright}docker stop $(docker ps -q)${colors.reset}`);
        log(`  • Remove all containers: ${colors.bright}docker-compose down --remove-orphans${colors.reset}`);
        process.exit(1);
    }
}

main();