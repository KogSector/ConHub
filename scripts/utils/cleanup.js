/**
 * Service cleanup script to ensure proper termination of npm processes
 */
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Create the .pid directory if it doesn't exist
const pidDir = path.join(__dirname, '..', '..', '.pid');
if (!fs.existsSync(pidDir)) {
  fs.mkdirSync(pidDir, { recursive: true });
}

// Function to kill processes by port
function killProcessOnPort(port) {
  try {
    // For Windows
    const command = `powershell -Command "Get-NetTCPConnection -LocalPort ${port} -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id (Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue).Id -Force -ErrorAction SilentlyContinue }"`;
    execSync(command);
    console.log(`Killed process on port ${port}`);
  } catch (error) {
    console.log(`No process found on port ${port}`);
  }
}

// Common development ports
const ports = [3000, 3001, 3002, 8000, 8080];

// Kill processes on common ports
ports.forEach(port => {
  killProcessOnPort(port);
});

// Clean up any PID files
const pidFiles = fs.readdirSync(pidDir);
pidFiles.forEach(file => {
  const pidPath = path.join(pidDir, file);
  try {
    const pid = fs.readFileSync(pidPath, 'utf8').trim();
    try {
      // For Windows
      execSync(`powershell -Command "Stop-Process -Id ${pid} -Force -ErrorAction SilentlyContinue"`);
      console.log(`Killed process with PID ${pid}`);
    } catch (error) {
      console.log(`Process with PID ${pid} not found or already terminated`);
    }
    fs.unlinkSync(pidPath);
  } catch (error) {
    console.error(`Error processing PID file ${file}:`, error.message);
  }
});