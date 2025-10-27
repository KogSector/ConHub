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

async function checkDockerRunning() {
    return new Promise((resolve) => {
        exec('docker info', (error) => {
            resolve(!error);
        });
    });
}

async function checkEnvFile() {
    const envPath = path.join(process.cwd(), '.env');
    const envExamplePath = path.join(process.cwd(), '.env.example');
    
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

async function stopExistingContainers() {
    try {
        logStep('CLEANUP', 'Stopping existing ConHub containers...');
        await runCommand('docker-compose down --remove-orphans');
        logSuccess('Existing containers stopped');
    } catch (error) {
        logWarning('No existing containers to stop or error occurred during cleanup');
    }
}

async function buildContainers() {
    try {
        logStep('BUILD', 'Building Docker containers...');
        await runCommand('docker-compose build --parallel');
        logSuccess('All containers built successfully');
    } catch (error) {
        logError('Failed to build containers');
        throw error;
    }
}

async function startContainers() {
    try {
        logStep('START', 'Starting all services...');
        await runCommand('docker-compose up -d');
        logSuccess('All services started successfully');
    } catch (error) {
        logError('Failed to start containers');
        throw error;
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
        
        log(`\n${colors.yellow}To stop all services: ${colors.bright}npm run docker:stop${colors.reset}`);
        log(`${colors.yellow}To view logs: ${colors.bright}docker-compose logs -f${colors.reset}`);
        log(`${colors.yellow}To view logs for specific service: ${colors.bright}docker-compose logs -f <service-name>${colors.reset}`);
        
    } catch (error) {
        logWarning('Could not retrieve status information');
    }
}

async function main() {
    try {
        log(`${colors.bright}${colors.magenta}🐳 ConHub Docker Startup Script${colors.reset}\n`);
        
        // Check if Docker is running
        logStep('CHECK', 'Verifying Docker is running...');
        const dockerRunning = await checkDockerRunning();
        if (!dockerRunning) {
            logError('Docker is not running. Please start Docker Desktop and try again.');
            process.exit(1);
        }
        logSuccess('Docker is running');

        // Check environment file
        logStep('ENV', 'Checking environment configuration...');
        const envExists = await checkEnvFile();
        if (!envExists) {
            process.exit(1);
        }
        logSuccess('Environment configuration found');

        // Stop existing containers
        await stopExistingContainers();

        // Build containers
        await buildContainers();

        // Start containers
        await startContainers();

        // Show status
        await showStatus();

    } catch (error) {
        logError(`Startup failed: ${error.message}`);
        log(`\n${colors.yellow}Troubleshooting tips:${colors.reset}`);
        log(`  • Make sure Docker Desktop is running`);
        log(`  • Check if ports are available (3000, 8000, 5432, 6379, 6333)`);
        log(`  • Verify .env file has correct values`);
        log(`  • Try running: ${colors.bright}docker-compose down --remove-orphans${colors.reset}`);
        log(`  • Check logs: ${colors.bright}docker-compose logs${colors.reset}`);
        process.exit(1);
    }
}

// Handle Ctrl+C gracefully
process.on('SIGINT', () => {
    log(`\n${colors.yellow}Received interrupt signal. Use 'npm run docker:stop' to stop services.${colors.reset}`);
    process.exit(0);
});

main();