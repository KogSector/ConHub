
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');


const pidDir = path.join(__dirname, '..', '.pid');
if (!fs.existsSync(pidDir)) {
  fs.mkdirSync(pidDir, { recursive: true });
}


function killProcessOnPort(port) {
  try {
    
    const command = `powershell -Command "Get-NetTCPConnection -LocalPort ${port} -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id (Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue).Id -Force -ErrorAction SilentlyContinue }" 2>$null`;
    execSync(command, { stdio: 'pipe' });
    console.log(`Killed process on port ${port}`);
  } catch (error) {
    console.log(`No process found on port ${port}`);
  }
}


const ports = [3000, 3001, 3002, 8000, 8080];


ports.forEach(port => {
  killProcessOnPort(port);
});


const pidFiles = fs.readdirSync(pidDir);
pidFiles.forEach(file => {
  const pidPath = path.join(pidDir, file);
  try {
    const pid = fs.readFileSync(pidPath, 'utf8').trim();
    try {
      
      execSync(`powershell -Command "Stop-Process -Id ${pid} -Force -ErrorAction SilentlyContinue"`, { stdio: 'pipe' });
      console.log(`Killed process with PID ${pid}`);
    } catch (error) {
      console.log(`Process with PID ${pid} not found or already terminated`);
    }
    fs.unlinkSync(pidPath);
  } catch (error) {
    console.error(`Error processing PID file ${file}:`, error.message);
  }
});

console.log('Cleanup completed successfully');