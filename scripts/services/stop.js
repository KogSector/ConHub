#!/usr/bin/env node

const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);

const SERVICES = {
  auth: { port: 3010, name: 'auth-service' },
  data: { port: 3013, name: 'data-service' },
  billing: { port: 3011, name: 'billing-service' },
  security: { port: 3014, name: 'security-service' },
  webhook: { port: 3015, name: 'webhook-service' },
  frontend: { port: 3000, name: 'next-dev' }
};

class ServiceStopper {
  async killProcessOnPort(port) {
    try {
      if (process.platform === 'win32') {
        // Windows
        const { stdout } = await execAsync(`netstat -ano | findstr :${port}`);
        const lines = stdout.split('\n').filter(line => line.includes('LISTENING'));
        
        for (const line of lines) {
          const parts = line.trim().split(/\s+/);
          const pid = parts[parts.length - 1];
          if (pid && pid !== '0') {
            try {
              await execAsync(`taskkill /F /PID ${pid}`);
              console.log(`✅ Killed process ${pid} on port ${port}`);
            } catch (error) {
              // Process might already be dead
            }
          }
        }
      } else {
        // Unix-like systems
        try {
          const { stdout } = await execAsync(`lsof -ti:${port}`);
          const pids = stdout.trim().split('\n').filter(pid => pid);
          
          for (const pid of pids) {
            try {
              await execAsync(`kill -9 ${pid}`);
              console.log(`✅ Killed process ${pid} on port ${port}`);
            } catch (error) {
              // Process might already be dead
            }
          }
        } catch (error) {
          // No processes found on port
        }
      }
    } catch (error) {
      // Port might not be in use
    }
  }

  async killProcessByName(name) {
    try {
      if (process.platform === 'win32') {
        // Windows
        await execAsync(`taskkill /F /IM "${name}.exe" 2>nul`);
      } else {
        // Unix-like systems
        await execAsync(`pkill -f ${name}`);
      }
      console.log(`✅ Killed ${name} processes`);
    } catch (error) {
      // Process might not be running
    }
  }

  async stopService(serviceName) {
    if (!SERVICES[serviceName]) {
      console.error(`❌ Unknown service: ${serviceName}`);
      return;
    }

    const service = SERVICES[serviceName];
    console.log(`🛑 Stopping ${serviceName} service...`);

    // Try to kill by port first
    await this.killProcessOnPort(service.port);
    
    // Also try to kill by process name
    await this.killProcessByName(service.name);
    
    console.log(`✅ ${serviceName} service stopped`);
  }

  async stopAll() {
    console.log('🛑 Stopping all ConHub services...');
    
    const serviceNames = Object.keys(SERVICES);
    
    // Stop all services in parallel
    await Promise.all(serviceNames.map(service => this.stopService(service)));
    
    // Additional cleanup for common process names
    const commonProcesses = ['cargo', 'node', 'npm'];
    for (const proc of commonProcesses) {
      try {
        if (process.platform === 'win32') {
          // Only kill ConHub-related processes
          await execAsync(`wmic process where "commandline like '%ConHub%' and name='${proc}.exe'" delete 2>nul`);
        }
      } catch (error) {
        // Ignore errors
      }
    }
    
    console.log('🎉 All services stopped!');
  }

  async stopSpecific(services) {
    console.log(`🛑 Stopping services: ${services.join(', ')}`);
    
    for (const service of services) {
      await this.stopService(service);
    }
  }
}

async function main() {
  const args = process.argv.slice(2);
  const serviceStopper = new ServiceStopper();

  if (args.length === 0 || args[0] === 'all') {
    await serviceStopper.stopAll();
  } else {
    await serviceStopper.stopSpecific(args);
  }
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = ServiceStopper;
