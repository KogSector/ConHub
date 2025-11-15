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
  const args = process.argv.slice(2);
  const projectRoot = path.resolve(__dirname, '..', '..');
  const cleanup = args.includes('--cleanup');
  const force = args.includes('--force');

  const downArgs = ['compose', 'down'];
  downArgs.push('--remove-orphans');
  if (cleanup) downArgs.push('-v');
  if (force) downArgs.push('--rmi', 'all', '-v');

  try {
    await run('docker', downArgs, projectRoot);
  } catch (e) {
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
