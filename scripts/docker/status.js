#!/usr/bin/env node

const { spawn, exec } = require('child_process');
const http = require('http');
const https = require('https');

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
        const child = spawn(command, [], {
            shell: true,
            cwd,
            stdio: 'pipe'
        });

        let stdout = '';
        let stderr = '';

        child.stdout.on('data', (data) => {
            stdout += data.toString();
        });

        child.stderr.on('data', (data) => {
            stderr += data.toString();
        });

        child.on('close', (code) => {
            if (code === 0) {
                resolve(stdout);
            } else {
                reject(new Error(`Command failed: ${stderr}`));
            }
        });

        child.on('error', (error) => {
            reject(error);
        });
    });
}

async function checkHttpEndpoint(url, timeout = 5000) {
    return new Promise((resolve) => {
        const protocol = url.startsWith('https') ? https : http;
        
        const req = protocol.get(url, (res) => {
            resolve({
                status: res.statusCode,
                success: res.statusCode >= 200 && res.statusCode < 400
            });
        });

        req.setTimeout(timeout, () => {
            req.destroy();
            resolve({ success: false, error: 'Timeout' });
        });

        req.on('error', (error) => {
            resolve({ success: false, error: error.message });
        });
    });
}

async function getContainerStatus() {
    try {
        const output = await runCommand('docker-compose ps --format json');
        const lines = output.trim().split('\n').filter(line => line.trim());
        return lines.map(line => JSON.parse(line));
    } catch (error) {
        return [];
    }
}

async function checkServiceHealth() {
    const services = [
        { name: 'Frontend', url: 'http://localhost:3000', container: 'conhub-frontend' },
        { name: 'API Gateway', url: 'http://localhost:80', container: 'conhub-nginx' },
        { name: 'Backend', url: 'http://localhost:8000/health', container: 'conhub-backend' },
        { name: 'Auth Service', url: 'http://localhost:3010/health', container: 'conhub-auth' },
        { name: 'Billing Service', url: 'http://localhost:3011/health', container: 'conhub-billing' },
        { name: 'Security Service', url: 'http://localhost:3012/health', container: 'conhub-security' },
        { name: 'Data Service', url: 'http://localhost:3013/health', container: 'conhub-data' },
        { name: 'AI Service', url: 'http://localhost:3014/health', container: 'conhub-ai' },
        { name: 'Webhook Service', url: 'http://localhost:3015/health', container: 'conhub-webhook' },
        { name: 'MCP Service', url: 'http://localhost:3004/health', container: 'conhub-mcp' }
    ];

    const infrastructure = [
        { name: 'PostgreSQL', url: 'http://localhost:5432', container: 'conhub-postgres', skipHttp: true },
        { name: 'Redis', url: 'http://localhost:6379', container: 'conhub-redis', skipHttp: true },
        { name: 'Qdrant', url: 'http://localhost:6333/health', container: 'conhub-qdrant' }
    ];

    log(`${colors.bright}${colors.cyan}🔍 ConHub Service Health Check${colors.reset}\n`);

    // Get container status
    const containers = await getContainerStatus();
    const containerMap = {};
    containers.forEach(container => {
        containerMap[container.Name] = container;
    });

    // Check application services
    log(`${colors.bright}Application Services:${colors.reset}`);
    for (const service of services) {
        const container = containerMap[service.container];
        if (!container) {
            logError(`${service.name.padEnd(20)} - Container not found`);
            continue;
        }

        const isRunning = container.State === 'running';
        if (!isRunning) {
            logError(`${service.name.padEnd(20)} - Container not running (${container.State})`);
            continue;
        }

        const health = await checkHttpEndpoint(service.url);
        if (health.success) {
            logSuccess(`${service.name.padEnd(20)} - Running and healthy`);
        } else {
            logWarning(`${service.name.padEnd(20)} - Running but health check failed (${health.error || 'Unknown error'})`);
        }
    }

    // Check infrastructure services
    log(`\n${colors.bright}Infrastructure Services:${colors.reset}`);
    for (const service of infrastructure) {
        const container = containerMap[service.container];
        if (!container) {
            logError(`${service.name.padEnd(20)} - Container not found`);
            continue;
        }

        const isRunning = container.State === 'running';
        if (!isRunning) {
            logError(`${service.name.padEnd(20)} - Container not running (${container.State})`);
            continue;
        }

        if (service.skipHttp) {
            logSuccess(`${service.name.padEnd(20)} - Running`);
        } else {
            const health = await checkHttpEndpoint(service.url);
            if (health.success) {
                logSuccess(`${service.name.padEnd(20)} - Running and healthy`);
            } else {
                logWarning(`${service.name.padEnd(20)} - Running but health check failed`);
            }
        }
    }

    // Show resource usage
    log(`\n${colors.bright}Resource Usage:${colors.reset}`);
    try {
        await runCommand('docker stats --no-stream --format "table {{.Container}}\\t{{.CPUPerc}}\\t{{.MemUsage}}"');
    } catch (error) {
        logWarning('Could not retrieve resource usage information');
    }
}

async function main() {
    try {
        await checkServiceHealth();
        
        log(`\n${colors.cyan}Commands:${colors.reset}`);
        log(`  • View logs: ${colors.bright}docker-compose logs -f${colors.reset}`);
        log(`  • Stop services: ${colors.bright}npm run docker:stop${colors.reset}`);
        log(`  • Restart services: ${colors.bright}npm start${colors.reset}`);
        
    } catch (error) {
        logError(`Status check failed: ${error.message}`);
        process.exit(1);
    }
}

main();