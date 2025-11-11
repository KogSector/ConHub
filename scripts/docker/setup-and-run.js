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

function readFeatureToggles() {
    // Resolve toggles from project root to match local start behavior
    const togglesPath = path.resolve(__dirname, '..', '..', 'feature-toggles.json');
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

function getAllowedServices(toggles) {
    const all = ['frontend','backend','auth','billing','security','data','client','webhook','nginx','postgres','redis','qdrant'];
    if (toggles && toggles.Heavy === false) {
        // Heavy=false should only skip compute-heavy services (embeddings/indexers).
        // Compose doesn't include embedding/indexers, so start the full stack.
        return all;
    }
    if (toggles && toggles.Auth === false) {
        return all.filter(s => !['auth','postgres','redis','qdrant'].includes(s));
    }
    return all;
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
    // Primary check: try 'docker info' which requires server connectivity
    try {
        await runCommand('docker info', process.cwd(), true);
        return { running: true };
    } catch (error) {
        // Secondary: gather diagnostic information from 'docker version' or the original error
        try {
            const versionOutput = await runCommand('docker version', process.cwd(), true);
            return { running: false, diagnostic: versionOutput.trim() };
        } catch (err2) {
            // Fall back to the original error message
            return { running: false, diagnostic: (error && error.message) ? error.message : String(error) };
        }
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
        return output.split('\n').filter(line => line.trim());
    } catch (error) {
        return [];
    }
}

function parseComposeDeclaredImages() {
    try {
        const projectRoot = path.resolve(__dirname, '..', '..');
        const composePath = path.join(projectRoot, 'docker-compose.yml');
        const content = fs.readFileSync(composePath, 'utf8');

        const lines = content.split(/\r?\n/);
        const images = [];
        const servicesWithBuild = new Set();

        let currentService = null;
        let inServices = false;
        for (let rawLine of lines) {
            const line = rawLine.replace(/\t/g, '    ');
            const trimmed = line.trim();
            if (!trimmed) continue;

            if (/^services:\s*$/.test(trimmed)) {
                inServices = true;
                continue;
            }

            if (!inServices) continue;

            // service definition (starts at column 2 or 0) - detect by pattern 'name:' at 2-space indent
            const serviceMatch = /^([a-zA-Z0-9_\-]+):\s*$/.exec(trimmed);
            if (serviceMatch && line.startsWith('  ')) {
                currentService = serviceMatch[1];
                continue;
            }

            if (!currentService) continue;

            const imageMatch = /^image:\s*(.+)$/.exec(trimmed);
            if (imageMatch) {
                let img = imageMatch[1].trim();
                // remove quotes
                img = img.replace(/^['"]|['"]$/g, '');
                images.push(img);
                continue;
            }

            const buildMatch = /^build:\s*(?:\n)?/.exec(trimmed);
            if (buildMatch) {
                servicesWithBuild.add(currentService);
                continue;
            }
        }

        return { images, servicesWithBuild: Array.from(servicesWithBuild) };
    } catch (error) {
        return { images: [], servicesWithBuild: [] };
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

async function buildContainers(forceBuild = false, toggles = { Auth: false }) {
    // Inspect docker-compose.yml to know which images are declared and which services have build contexts
    const { images: declaredImages, servicesWithBuild } = parseComposeDeclaredImages();

    const existingImages = await getExistingImages();
    const existingSet = new Set(existingImages.map(i => i.toLowerCase()));

    // Determine which declared images are missing locally
    const missingDeclared = declaredImages.filter(img => !existingSet.has(img.toLowerCase()));

    // If any service has a build context, or any declared image is missing, we should build (unless forceBuild false and nothing missing)
    const shouldBuild = forceBuild || missingDeclared.length > 0 || servicesWithBuild.length > 0 && declaredImages.length === 0;

    if (!shouldBuild) {
        logInfo('All declared images are present locally. Skipping build...');
        logInfo('Use --force-build flag to rebuild images');
        return;
    }

    try {
        logStep('BUILD', forceBuild ? 'Force rebuilding Docker containers...' : 'Building Docker containers...');

        if (missingDeclared.length > 0) {
            logInfo('Missing images detected:');
            missingDeclared.forEach(i => log(`  • ${i}`));
        }
        if (servicesWithBuild.length > 0) {
            logInfo('Services with build contexts detected:');
            servicesWithBuild.forEach(s => log(`  • ${s}`));
        }

        const projectRoot = path.resolve(__dirname, '..', '..');
        const allowed = getAllowedServices(toggles);
        const buildCmd = allowed.length > 0 ? `docker-compose build --parallel ${allowed.join(' ')}` : 'docker-compose build --parallel';
        await runCommand(buildCmd, projectRoot);
        logSuccess('All containers built successfully');
    } catch (error) {
        logError('Failed to build containers');
        throw error;
    }
}

async function startContainers(toggles = { Auth: false }) {
    try {
        logStep('START', 'Starting ConHub services...');
        // Run docker-compose from project root directory
        const projectRoot = path.resolve(__dirname, '..', '..');
        const allowed = getAllowedServices(toggles);
        const upCmd = allowed.length > 0 ? `docker-compose up -d --no-deps ${allowed.join(' ')}` : 'docker-compose up -d';
        await runCommand(upCmd, projectRoot);
        if (toggles && toggles.Heavy === false) {
            logInfo('Heavy=false — skipping embeddings and indexers (not in compose)');
        } else if (toggles && toggles.Auth === false) {
            logInfo('Auth=false — auth and database services are disabled');
        }
        logSuccess('Services start command executed');
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

async function showStatus(toggles = { Auth: false }) {
    try {
        logStep('STATUS', 'Checking service status...');
        await runCommand('docker-compose ps');
        
        log(`\n${colors.bright}🚀 ConHub is now running!${colors.reset}`);
        log(`\n${colors.cyan}Available services:${colors.reset}`);
        log(`  • Frontend: ${colors.green}http://localhost:3000${colors.reset}`);
        if (toggles && toggles.Heavy === false) {
            // Heavy=false: core services remain running; embeddings/indexers are skipped
            log(`  • API Gateway: ${colors.green}http://localhost:80${colors.reset}`);
            log(`  • Backend: ${colors.green}http://localhost:8000${colors.reset}`);
            if (!(toggles && toggles.Auth === false)) {
                log(`  • Auth Service: ${colors.green}http://localhost:3010${colors.reset}`);
            } else {
                log(`  • Auth Service: ${colors.yellow}disabled via feature toggles${colors.reset}`);
            }
            log(`  • Billing Service: ${colors.green}http://localhost:3011${colors.reset}`);
            log(`  • Security Service: ${colors.green}http://localhost:3012${colors.reset}`);
            log(`  • Data Service: ${colors.green}http://localhost:3013${colors.reset}`);
            log(`  • AI Service: ${colors.green}http://localhost:3014${colors.reset}`);
            log(`  • Webhook Service: ${colors.green}http://localhost:3015${colors.reset}`);
            log(`  • MCP Service: ${colors.green}http://localhost:3004${colors.reset}`);
            log(`  • Embeddings: ${colors.yellow}disabled (Heavy=false)${colors.reset}`);
            log(`  • Indexers: ${colors.yellow}disabled (Heavy=false)${colors.reset}`);
        } else {
            log(`  • API Gateway: ${colors.green}http://localhost:80${colors.reset}`);
            log(`  • Backend: ${colors.green}http://localhost:8000${colors.reset}`);
            if (!(toggles && toggles.Auth === false)) {
                log(`  • Auth Service: ${colors.green}http://localhost:3010${colors.reset}`);
            } else {
                log(`  • Auth Service: ${colors.yellow}disabled via feature toggles${colors.reset}`);
            }
            log(`  • Billing Service: ${colors.green}http://localhost:3011${colors.reset}`);
            log(`  • Security Service: ${colors.green}http://localhost:3012${colors.reset}`);
            log(`  • Data Service: ${colors.green}http://localhost:3013${colors.reset}`);
            log(`  • AI Service: ${colors.green}http://localhost:3014${colors.reset}`);
            log(`  • Webhook Service: ${colors.green}http://localhost:3015${colors.reset}`);
            log(`  • MCP Service: ${colors.green}http://localhost:3004${colors.reset}`);
        }
        
        log(`\n${colors.cyan}Infrastructure:${colors.reset}`);
        if (!(toggles && toggles.Auth === false)) {
            log(`  • PostgreSQL: ${colors.green}localhost:5432${colors.reset}`);
            log(`  • Redis: ${colors.green}localhost:6379${colors.reset}`);
            log(`  • Qdrant: ${colors.green}localhost:6333${colors.reset}`);
        } else {
            log(`  • Databases: ${colors.yellow}disabled via feature toggles (Postgres/Redis/Qdrant)${colors.reset}`);
        }
        
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
    const startOnly = args.includes('--start-only');
    const helpRequested = args.includes('--help') || args.includes('-h');
    
    if (helpRequested) {
        log(`${colors.bright}${colors.magenta}🐳 ConHub Docker Setup & Run Script${colors.reset}\n`);
        log(`${colors.cyan}Description:${colors.reset}`);
        log(`  Intelligently sets up and runs ConHub with Docker.`);
        log(`  - Builds containers only if they don't exist`);
        log(`  - Runs existing containers if they're already built`);
        log(`  - Handles environment setup automatically\n`);
        
        log(`${colors.cyan}Usage:${colors.reset}`);
        log(`  npm run docker:setup                  # Setup and run (build only if needed)`);
        log(`  npm run docker:start                  # Start services without rebuilding existing images`);
        log(`  npm run docker:setup -- --force-build # Force rebuild all containers`);
        log(`  npm run docker:setup -- --help        # Show this help\n`);
        
        log(`${colors.cyan}Flags:${colors.reset}`);
        log(`  --force-build, -f    Force rebuild all containers even if they exist`);
        log(`  --start-only         Skip container teardown and reuse existing images when available`);
        log(`  --help, -h           Show this help message`);
        
        return;
    }
    
    try {
        log(`${colors.bright}${colors.magenta}🐳 ConHub Docker Setup & Run Script${colors.reset}\n`);
        const toggles = readFeatureToggles();
        if (toggles.Heavy === false) {
            logInfo('Feature toggles: Heavy=false — will skip embeddings and indexers');
        } else if (toggles.Auth === false) {
            logInfo('Feature toggles: Auth=false — will skip building/starting auth and databases');
        }
        
        // Check if Docker is running (with diagnostics)
        logStep('DOCKER', 'Verifying Docker is running...');
        const dockerCheck = await checkDockerRunning();
        if (!dockerCheck.running) {
            logError('Docker is not running. Please start Docker Desktop and try again.');
            if (dockerCheck.diagnostic) {
                logInfo('Docker diagnostic output:');
                // Print diagnostic output indented for readability
                dockerCheck.diagnostic.split('\n').forEach(line => {
                    if (line.trim()) log(`  ${line}`);
                });
            }
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

        if (startOnly) {
            logStep('CLEANUP', 'Start-only mode detected; skipping container teardown');
        } else {
            await stopRunningContainers();
        }

        await buildContainers(forceBuild, toggles);

        // Start containers
        await startContainers(toggles);

        // Wait for services to be ready
        await waitForServices();

        // Show status
        await showStatus(toggles);

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