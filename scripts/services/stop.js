#!/usr/bin/env node

const { exec } = require('child_process');
const util = require('util');
const execAsync = util.promisify(exec);

const PORTS = [3010, 3011, 3012, 3013, 3014, 3015, 8000, 8080, 8082, 3000];

async function killProcessOnPort(port) {
  try {
    console.log(`🔍 Checking port ${port}...`);
    
    // Find process using the port
    const { stdout } = await execAsync(`netstat -ano | findstr :${port}`);
    
    if (stdout.trim()) {
      const lines = stdout.trim().split('\n');
      const pids = new Set();
      
      for (const line of lines) {
        const parts = line.trim().split(/\s+/);
        if (parts.length >= 5 && parts[1].includes(`:${port}`)) {
          const pid = parts[4];
          if (pid && pid !== '0') {
            pids.add(pid);
          }
        }
      }
      
      for (const pid of pids) {
        try {
          console.log(`🛑 Killing process ${pid} on port ${port}...`);
          await execAsync(`taskkill /F /PID ${pid}`);
          console.log(`✅ Process ${pid} killed`);
        } catch (error) {
          console.log(`⚠️  Could not kill process ${pid}: ${error.message}`);
        }
      }
    } else {
      console.log(`✅ Port ${port} is free`);
    }
  } catch (error) {
    console.log(`✅ Port ${port} is free (no processes found)`);
  }
}

async function stopAllServices() {
  console.log('🛑 ConHub Service Stopper');
  console.log('='.repeat(50));
  
  console.log('🔍 Stopping all ConHub services...');
  
  for (const port of PORTS) {
    await killProcessOnPort(port);
  }
  
  // Also kill any remaining cargo/node processes that might be ConHub related
  try {
    console.log('🧹 Cleaning up any remaining ConHub processes...');
    
    // Kill any remaining cargo processes
    try {
      await execAsync('taskkill /F /IM "cargo.exe" 2>nul');
      console.log('✅ Stopped cargo processes');
    } catch (e) {
      // Ignore if no cargo processes
    }
    
    // Kill any ConHub-related node processes
    try {
      const { stdout } = await execAsync('wmic process where "name=\'node.exe\'" get processid,commandline /format:csv');
      const lines = stdout.split('\n');
      
      for (const line of lines) {
        if (line.includes('ConHub') || line.includes('conhub')) {
          const parts = line.split(',');
          if (parts.length >= 3) {
            const pid = parts[2].trim();
            if (pid && pid !== 'ProcessId') {
              try {
                await execAsync(`taskkill /F /PID ${pid}`);
                console.log(`✅ Stopped ConHub node process ${pid}`);
              } catch (e) {
                // Ignore
              }
            }
          }
        }
      }
    } catch (e) {
      // Ignore if no node processes
    }
    
  } catch (error) {
    console.log('⚠️  Some cleanup operations failed, but main services should be stopped');
  }
  
  console.log('✅ All ConHub services stopped');
  console.log('🚀 You can now run "npm start" to restart services');
}

if (require.main === module) {
  stopAllServices().catch(console.error);
}

module.exports = { stopAllServices, killProcessOnPort };