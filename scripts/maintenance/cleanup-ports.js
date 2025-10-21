#!/usr/bin/env node

const { execSync } = require('child_process');
const os = require('os');
const fs = require('fs');

const colors = {
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

const PORTS = [3000, 3001, 3002, 8001, 8003];
const isWindows = os.platform() === 'win32';

PORTS.forEach(port => {
  console.log(`${colors.yellow}Checking port ${port}...${colors.reset}`);

  try {
    let pid;

    if (isWindows) {
      // Windows
      const output = execSync(`netstat -ano | findstr :${port}`, { encoding: 'utf8' });
      const match = output.match(/\s+(\d+)\s*$/m);
      if (match) {
        pid = match[1];
        console.log(`${colors.yellow}Port ${port} is in use by PID ${pid}. Terminating process...${colors.reset}`);
        execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
      }
    } else {
      // Unix-like (Linux, macOS)
      try {
        pid = execSync(`lsof -t -i:${port}`, { encoding: 'utf8' }).trim();
        if (pid) {
          console.log(`${colors.yellow}Port ${port} is in use by PID ${pid}. Terminating process...${colors.reset}`);
          execSync(`kill -9 ${pid}`, { stdio: 'ignore' });
        }
      } catch (e) {
        // Port not in use or lsof not available
      }
    }
  } catch (error) {
    // Port not in use or command not available
  }
});

// Clean up lock file
const lockFile = 'lexor_data/index/.tantivy-writer.lock';
if (fs.existsSync(lockFile)) {
  console.log(`${colors.yellow}Found lock file at ${lockFile}. Removing...${colors.reset}`);
  fs.unlinkSync(lockFile);
}

// Give the OS a moment to release resources
setTimeout(() => {
  console.log('Cleanup complete.');
  process.exit(0);
}, 3000);
