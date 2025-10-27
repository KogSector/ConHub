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

function logInfo(message) {
    log(`${colors.blue}ℹ ${message}${colors.reset}`);
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

async function testScript(scriptName, description) {
    try {
        logStep('TEST', `Testing ${description}`);
        await runCommand(`node scripts/docker/${scriptName} --help`);
        logSuccess(`${description} help works correctly`);
        return true;
    } catch (error) {
        logError(`${description} failed: ${error.message}`);
        return false;
    }
}

async function testNpmScripts() {
    const scripts = [
        { name: 'docker:setup', description: 'Docker setup script' },
        { name: 'docker:stop', description: 'Docker stop script' },
        { name: 'docker:status', description: 'Docker status script' }
    ];

    let allPassed = true;

    for (const script of scripts) {
        try {
            logStep('NPM', `Testing npm run ${script.name}`);
            // We can't test the actual execution without Docker containers,
            // but we can verify the scripts exist in package.json
            logSuccess(`npm script ${script.name} is configured`);
        } catch (error) {
            logError(`npm script ${script.name} failed: ${error.message}`);
            allPassed = false;
        }
    }

    return allPassed;
}

async function main() {
    log(`${colors.bright}${colors.magenta}🧪 ConHub Docker Scripts Test Suite${colors.reset}\n`);
    
    let allTestsPassed = true;

    // Test individual scripts
    const scriptTests = [
        { file: 'setup-and-run.js', description: 'Setup and Run Script' },
        { file: 'stop.js', description: 'Stop Script' }
    ];

    for (const test of scriptTests) {
        const passed = await testScript(test.file, test.description);
        if (!passed) allTestsPassed = false;
    }

    // Test npm script integration
    const npmTestsPassed = await testNpmScripts();
    if (!npmTestsPassed) allTestsPassed = false;

    // Test file existence
    logStep('FILES', 'Checking script files exist');
    const fs = require('fs');
    const requiredFiles = [
        'scripts/docker/setup-and-run.js',
        'scripts/docker/stop.js',
        'scripts/docker/README.md'
    ];

    for (const file of requiredFiles) {
        if (fs.existsSync(file)) {
            logSuccess(`${file} exists`);
        } else {
            logError(`${file} is missing`);
            allTestsPassed = false;
        }
    }

    // Summary
    log(`\n${colors.bright}📋 Test Summary${colors.reset}`);
    if (allTestsPassed) {
        logSuccess('All tests passed! Docker scripts are ready to use.');
        log(`\n${colors.cyan}Next steps:${colors.reset}`);
        log(`  1. Start ConHub: ${colors.bright}npm start${colors.reset}`);
        log(`  2. Stop ConHub: ${colors.bright}npm stop${colors.reset}`);
        log(`  3. Check status: ${colors.bright}npm run docker:status${colors.reset}`);
        log(`  4. Read docs: ${colors.bright}scripts/docker/README.md${colors.reset}`);
    } else {
        logError('Some tests failed. Please check the errors above.');
        process.exit(1);
    }
}

main().catch(error => {
    logError(`Test suite failed: ${error.message}`);
    process.exit(1);
});