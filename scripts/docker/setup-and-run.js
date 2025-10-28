#!/usr/bin/env node

const { spawn, exec } = require('child_process');
const path = require('path');
const fs = require('fs');

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

async function checkDockerComposeExists() {
    // Look for docker-compose.yml in the project root (parent of scripts directory)
    const projectRoot = path.resolve(__dirname, '..', '..');
    const composePath = path.join(projectRoot, 'docker-compose.yml');
    return fs.existsSync(composePath);
}

async function getExistingImages() {
    try {
        const output = await runCommand('docker images --format "{{.Repository}}:{{.Tag}}"', process.cwd(), true);
        return output.split('\n').filter(line => line.trim() && line.includes('conhub'));
    } catch (error) {
        return [];
    }
}

async function getRunningContainers() {
    try {
        const output = await runCommand('docker-compose ps --services --filter "status=running"', process.cwd(), true);
        return output.split('\n').filter(line => line.trim());
    } catch (error) {
        return [];
    }
}

async function getAllContainers() {
    try {
        const output = await runCommand('docker-compose ps --services', process.cwd(), true);
        return output.split('\n').filter(line => line.trim());
    } catch (error) {
        return [];
    }
}

async function checkEnvFile() {
    // Look for .env files in the project root (parent of scripts directory)
    const projectRoot = path.resolve(__dirname, '..', '..');
    const envPath = path.join(projectRoot, '.env');
    const envExamplePath = path.join(projectRoot, '.env.example');
    
    if (!fs.existsSync(envPath)) {
        if (fs.existsSync(envExamplePath)) {
            logWarning('.env file not found. Copying from .env.example...');
            fs.copyFileSync(envExamplePath, envPath);
            logSuccess('.env file created from .env.example');
        } else {
            logError('.env file not found and .env.example does not exist');
            return false;
        }
    }
    
    // Update ENV_MODE to 'docker' for Docker deployment
    let envContent = fs.readFileSync(envPath, 'utf8');
    if (envContent.includes('ENV_MODE=local')) {
        envContent = envContent.replace('ENV_MODE=local', 'ENV_MODE=docker');
        fs.writeFileSync(envPath, envContent);
        logSuccess('Updated ENV_MODE to docker for container deployment');
    } else if (!envContent.includes('ENV_MODE=docker')) {
        // Add ENV_MODE if it doesn't exist
        envContent = 'ENV_MODE=docker\n' + envContent;
        fs.writeFileSync(envPath, envContent);
        logSuccess('Added ENV_MODE=docker for container deployment');
    }
    
    return true;
}

async function stopRunningContainers() {
    try {
        logStep('CLEANUP', 'Stopping any running ConHub containers...');
        // Run docker-compose from project root directory
        const projectRoot = path.resolve(__dirname, '..', '..');
        await runCommand('docker-compose down', projectRoot, true);
        logSuccess('Existing containers stopped');
    } catch (error) {
        logInfo('No running containers to stop');
    }
}

async function buildContainers(forceBuild = false) {
    const existingImages = await getExistingImages();
    const hasConHubImages = existingImages.length > 0;
    
    if (!forceBuild && hasConHubImages) {
        logInfo('ConHub Docker images already exist. Skipping build...');
        logInfo('Use --force-build flag to rebuild images');
        return;
    }
    
    try {
        logStep('BUILD', hasConHubImages ? 'Rebuilding Docker containers...' : 'Building Docker containers for the first time...');
        
        if (!hasConHubImages) {
            logInfo('This is your first time setting up ConHub with Docker.');
            logInfo('Building all containers... This may take several minutes.');
        }
        
        // Run docker-compose from project root directory
        const projectRoot = path.resolve(__dirname, '..', '..');
        await runCommand('docker-compose build --parallel', projectRoot);
        logSuccess('All containers built successfully');
    } catch (error) {
        logError('Failed to build containers');
        throw error;
    }
}

async function startContainers() {
    try {
        logStep('START', 'Starting all ConHub services...');
        // Run docker-compose from project root directory
        const projectRoot = path.resolve(__dirname, '..', '..');
        await runCommand('docker-compose up -d', projectRoot);
        logSuccess('All services started successfully');
    } catch (error) {
        logError('Failed to start containers');
        throw error;
    }
}

async function waitForServices() {
    logStep('HEALTH', 'Waiting for services to be ready...');
    
    // Wait a bit for services to start
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    try {
        await runCommand('docker-compose ps');
        logSuccess('Services are starting up');
    } catch (error) {
        logWarning('Could not check service status');
    }
}

async function showStatus() {
    try {
        logStep('STATUS', 'Checking service status...');
        await runCommand('docker-compose ps');
        
        log(`\n${colors.bright}🚀 ConHub is now running!${colors.reset}`);
        log(`\n${colors.cyan}Available services:${colors.reset}`);
        log(`  • Frontend: ${colors.green}http://localhost:3000${colors.reset}`);
        log(`  • API Gateway: ${colors.green}http://localhost:80${colors.reset}`);
        log(`  • Backend: ${colors.green}http://localhost:8000${colors.reset}`);
        log(`  • Auth Service: ${colors.green}http://localhost:3010${colors.reset}`);
        log(`  • Billing Service: ${colors.green}http://localhost:3011${colors.reset}`);
        log(`  • Security Service: ${colors.green}http://localhost:3012${colors.reset}`);
        log(`  • Data Service: ${colors.green}http://localhost:3013${colors.reset}`);
        log(`  • AI Service: ${colors.green}http://localhost:3014${colors.reset}`);
        log(`  • Webhook Service: ${colors.green}http://localhost:3015${colors.reset}`);
        log(`  • MCP Service: ${colors.green}http://localhost:3004${colors.reset}`);
        
        log(`\n${colors.cyan}Infrastructure:${colors.reset}`);
        log(`  • PostgreSQL: ${colors.green}localhost:5432${colors.reset}`);
        log(`  • Redis: ${colors.green}localhost:6379${colors.reset}`);
        log(`  • Qdrant: ${colors.green}localhost:6333${colors.reset}`);
        
        log(`\n${colors.yellow}Useful commands:${colors.reset}`);
        log(`  • Stop all services: ${colors.bright}npm run docker:stop${colors.reset}`);
        log(`  • View logs: ${colors.bright}docker-compose logs -f${colors.reset}`);
        log(`  • View logs for specific service: ${colors.bright}docker-compose logs -f <service-name>${colors.reset}`);
        log(`  • Rebuild containers: ${colors.bright}npm run docker:setup -- --force-build${colors.reset}`);
        
    } catch (error) {
        logWarning('Could not retrieve status information');
    }
}

async function main() {
    const args = process.argv.slice(2);
    const forceBuild = args.includes('--force-build') || args.includes('-f');
    const helpRequested = args.includes('--help') || args.includes('-h');
    
    if (helpRequested) {
        log(`${colors.bright}${colors.magenta}🐳 ConHub Docker Setup & Run Script${colors.reset}\n`);
        log(`${colors.cyan}Description:${colors.reset}`);
        log(`  Intelligently sets up and runs ConHub with Docker.`);
        log(`  - Builds containers only if they don't exist`);
        log(`  - Runs existing containers if they're already built`);
        log(`  - Handles environment setup automatically\n`);
        
        log(`${colors.cyan}Usage:${colors.reset}`);
        log(`  npm run docker:setup              # Setup and run (build only if needed)`);
        log(`  npm run docker:setup -- --force-build  # Force rebuild all containers`);
        log(`  npm run docker:setup -- --help         # Show this help\n`);
        
        log(`${colors.cyan}Flags:${colors.reset}`);
        log(`  --force-build, -f    Force rebuild all containers even if they exist`);
        log(`  --help, -h           Show this help message`);
        
        return;
    }
    
    try {
        log(`${colors.bright}${colors.magenta}🐳 ConHub Docker Setup & Run Script${colors.reset}\n`);
        
        // Check if Docker is running
        logStep('DOCKER', 'Verifying Docker is running...');
        const dockerRunning = await checkDockerRunning();
        if (!dockerRunning) {
            logError('Docker is not running. Please start Docker Desktop and try again.');
            logInfo('Make sure Docker Desktop is installed and running before proceeding.');
            process.exit(1);
        }
        logSuccess('Docker is running');

        // Check if docker-compose.yml exists
        logStep('COMPOSE', 'Checking docker-compose configuration...');
        const composeExists = await checkDockerComposeExists();
        if (!composeExists) {
            logError('docker-compose.yml not found in current directory');
            process.exit(1);
        }
        logSuccess('docker-compose.yml found');

        // Check environment file
        logStep('ENV', 'Checking environment configuration...');
        const envExists = await checkEnvFile();
        if (!envExists) {
            process.exit(1);
        }
        logSuccess('Environment configuration ready');

        // Stop any running containers
        await stopRunningContainers();

        // Build containers (only if needed or forced)
        await buildContainers(forceBuild);

        // Start containers
        await startContainers();

        // Wait for services to be ready
        await waitForServices();

        // Show status
        await showStatus();

    } catch (error) {
        logError(`Setup failed: ${error.message}`);
        log(`\n${colors.yellow}Troubleshooting tips:${colors.reset}`);
        log(`  • Make sure Docker Desktop is running`);
        log(`  • Check if ports are available (3000, 8000, 5432, 6379, 6333)`);
        log(`  • Verify .env file has correct values`);
        log(`  • Try running: ${colors.bright}docker-compose down --remove-orphans${colors.reset}`);
        log(`  • Check logs: ${colors.bright}docker-compose logs${colors.reset}`);
        log(`  • Force rebuild: ${colors.bright}npm run docker:setup -- --force-build${colors.reset}`);
        process.exit(1);
    }
}

// Handle Ctrl+C gracefully
process.on('SIGINT', () => {
    log(`\n${colors.yellow}Received interrupt signal. Use 'npm run docker:stop' to stop services.${colors.reset}`);
    process.exit(0);
});

main();