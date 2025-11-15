#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

function run(cmd, args, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { cwd, stdio: 'inherit', shell: true });
    child.on('close', (code) => {
      code === 0 ? resolve() : reject(new Error(`Exit code ${code}`));
    });
    child.on('error', reject);
  });
}

async function main() {
  const projectRoot = path.resolve(__dirname, '..', '..');
  try {
    await run('docker', ['compose', 'ps'], projectRoot);
  } catch (e) {
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
