#!/usr/bin/env node
// Platform detection script to run appropriate startup script

const { spawn } = require('child_process');
const os = require('os');
const path = require('path');

const command = process.argv[2] || 'start'; // Default to 'start' if no command provided

console.log(`🚀 ConHub - Detecting platform and running ${command}...\n`);

const platform = os.platform();

let scriptPath;
let execCommand;
let args = [];

if (platform === 'win32') {
    // Windows
    console.log('🖥️  Windows detected - Using PowerShell script');
    scriptPath = path.join(__dirname, `${command}.ps1`);
    execCommand = 'powershell';
    args = ['-ExecutionPolicy', 'Bypass', '-File', scriptPath];
} else {
    // Linux/macOS
    console.log('🐧 Unix-like system detected - Using shell script');
    scriptPath = path.join(__dirname, `${command}.sh`);
    execCommand = 'bash';
    args = [scriptPath];
}

console.log(`🔄 Executing: ${execCommand} ${args.join(' ')}\n`);

// Execute the appropriate script
const child = spawn(execCommand, args, {
    stdio: 'inherit',
    cwd: path.join(__dirname, '..') // Go back to project root
});

child.on('error', (error) => {
    console.error(`❌ Error running ConHub ${command}:`, error.message);
    process.exit(1);
});

child.on('close', (code) => {
    if (code !== 0) {
        console.error(`❌ ConHub ${command} failed with exit code ${code}`);
        process.exit(code);
    }
});