#!/usr/bin/env node


const { spawn, execSync } = require('child_process');
const os = require('os');
const path = require('path');

const command = process.argv[2] || 'start'; 

console.log(`🚀 ConHub - Detecting platform and running ${command}...\n`);

const platform = os.platform();
const projectRoot = path.join(__dirname, '..');

if (command === 'start') {
    
    try {
        
        console.log('Ensuring all services are stopped before starting...');
        if (platform === 'win32') {
            execSync(`powershell -ExecutionPolicy Bypass -File "${path.join(__dirname, 'stop.ps1')}"`, { stdio: 'inherit', cwd: projectRoot });
        } else {
            execSync(`bash "${path.join(__dirname, 'stop.sh')}"`, { stdio: 'inherit', cwd: projectRoot });
        }
        console.log('All services stopped.\n');
        
        
        console.log('Starting service monitor...');
        spawn('node', [path.join(__dirname, 'monitor-services.js')], {
            detached: true,
            stdio: 'ignore',
            cwd: projectRoot
        }).unref();

        
        console.log('Starting all services with concurrently...');
        const concurrentlyPath = path.join(projectRoot, 'node_modules', '.bin', platform === 'win32' ? 'concurrently.cmd' : 'concurrently');
        const child = spawn(
            concurrentlyPath,
            [
                '--kill-others',
                '--names',
                'Frontend,Backend,Lexor,AI',
                '--prefix-colors',
                'cyan,green,yellow,magenta',
                '--restart-tries',
                '3',
                'npm:dev:frontend',
                'npm:dev:backend',
                'npm:dev:lexor',
                'npm:dev:ai'
            ],
            {
                stdio: 'inherit',
                cwd: projectRoot
            }
        );

        child.on('error', (error) => {
            console.error(`❌ Error running ConHub start:`, error.message);
            process.exit(1);
        });

        child.on('close', (code) => {
            if (code !== 0) {
                console.error(`❌ ConHub start failed with exit code ${code}`);
            }
            console.log('ConHub services have stopped.');
            process.exit(code);
        });

    } catch (error) {
        console.error(`❌ An error occurred during the start process:`, error.message);
        process.exit(1);
    }

} else {
    
    let scriptPath;
    let execCommand;
    let args = [];

    if (platform === 'win32') {
        
        console.log('🖥️  Windows detected - Using PowerShell script');
        scriptPath = path.join(__dirname, `${command}.ps1`);
        execCommand = 'powershell';
        args = ['-ExecutionPolicy', 'Bypass', '-File', scriptPath];
    } else {
        
        console.log('🐧 Unix-like system detected - Using shell script');
        scriptPath = path.join(__dirname, `${command}.sh`);
        execCommand = 'bash';
        args = [scriptPath];
    }

    console.log(`🔄 Executing: ${execCommand} ${args.join(' ')}\n`);

    
    const child = spawn(execCommand, args, {
        stdio: 'inherit',
        cwd: projectRoot
    });

    child.on('error', (error) => {
        console.error(`❌ Error running ConHub ${command}:`, error.message);
        process.exit(1);
    });

    child.on('close', (code) => {
        if (code !== 0) {
            console.error(`❌ ConHub ${command} failed with exit code ${code}`);
        }
    });
}
