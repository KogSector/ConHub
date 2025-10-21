#!/usr/bin/env node

const { execSync } = require('child_process');
const os = require('os');

const PROCESSES = ['node', 'next-server', 'conhub-backend', 'lexor', 'python', 'uvicorn'];
const PORTS = [3000, 3001, 3002, 8001, 8003];
const isWindows = os.platform() === 'win32';

console.log('Force stopping ConHub related processes...');

// Kill processes by name
PROCESSES.forEach(procName => {
  try {
    if (isWindows) {
      // Windows - filter by process name
      const output = execSync(`tasklist /FI "IMAGENAME eq ${procName}*" /FO CSV /NH`, { encoding: 'utf8' });
      const lines = output.split('\n').filter(line => line.trim());

      lines.forEach(line => {
        const match = line.match(/"([^"]+)","(\d+)"/);
        if (match) {
          const pid = match[2];
          console.log(`Killing process ${procName} with PID ${pid}`);
          try {
            execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
          } catch (e) {}
        }
      });
    } else {
      // Unix-like (Linux, macOS)
      const pids = execSync(`pgrep -f "${procName}"`, { encoding: 'utf8' }).trim().split('\n').filter(Boolean);

      pids.forEach(pid => {
        try {
          const cmdline = execSync(`ps -p ${pid} -o command=`, { encoding: 'utf8' });
          // Check if command line contains relevant patterns
          if (/3000|3001|3002|8001|8003|conhub|lexor|uvicorn/.test(cmdline)) {
            console.log(`Killing process ${procName} with PID ${pid}`);
            execSync(`kill -9 ${pid}`, { stdio: 'ignore' });
          }
        } catch (e) {}
      });
    }
  } catch (error) {
    // Process not found
  }
});

// Kill processes by port
PORTS.forEach(port => {
  try {
    if (isWindows) {
      const output = execSync(`netstat -ano | findstr :${port}`, { encoding: 'utf8' });
      const match = output.match(/\s+(\d+)\s*$/m);
      if (match) {
        const pid = match[1];
        console.log(`Killing process on port ${port} with PID ${pid}`);
        execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
      }
    } else {
      const pids = execSync(`lsof -t -i:${port}`, { encoding: 'utf8' }).trim().split('\n').filter(Boolean);
      pids.forEach(pid => {
        console.log(`Killing process on port ${port} with PID ${pid}`);
        execSync(`kill -9 ${pid}`, { stdio: 'ignore' });
      });
    }
  } catch (error) {
    // Port not in use or command not available
  }
});

console.log('Force stop complete.');
