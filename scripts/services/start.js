#!/usr/bin/env node

const { spawn } = require('child_process');
const http = require('http');
const path = require('path');
const fs = require('fs');

const SERVICES = {
  auth: { port: 3010, path: 'auth', command: 'cargo', args: ['run'], healthPath: '/health' },
  data: { port: 3013, path: 'data', command: 'cargo', args: ['run'], healthPath: '/health' },
  billing: { port: 3011, path: 'billing', command: 'cargo', args: ['run'], healthPath: '/health' },
  security: { port: 3014, path: 'security', command: 'cargo', args: ['run'], healthPath: '/health' },
  webhook: { port: 3015, path: 'webhook', command: 'cargo', args: ['run'], healthPath: '/health' },
  frontend: { port: 3000, path: 'frontend', command: 'npm', args: ['run', 'dev'], healthPath: '/' }
};

class ServiceManager {
  constructor() {
    this.projectRoot = path.resolve(__dirname, '../..');
    this.processes = new Map();
  }

  async checkPrerequisites() {
    console.log('🔍 Checking prerequisites...');
    
    // Check if .env file exists
    const envPath = path.join(this.projectRoot, '.env');
    if (!fs.existsSync(envPath)) {
      console.log('⚠️  .env file not found, using defaults');
    } else {
      console.log('✅ .env file found');
    }

    // Check if database connection string is set
    if (!process.env.DATABASE_URL_NEON && !process.env.DATABASE_URL) {
      console.log('⚠️  No database connection string found in environment');
      console.log('   Set DATABASE_URL_NEON or DATABASE_URL in your .env file');
    } else {
      console.log('✅ Database connection string configured');
    }

    // Check if required directories exist
    const requiredDirs = ['auth', 'data', 'frontend'];
    for (const dir of requiredDirs) {
      const dirPath = path.join(this.projectRoot, dir);
      if (!fs.existsSync(dirPath)) {
        console.error(`❌ Required directory missing: ${dir}`);
        return false;
      }
    }

    console.log('✅ All prerequisites checked');
    return true;
  }

  async checkServiceHealth(serviceName) {
    const service = SERVICES[serviceName];
    if (!service) return false;

    return new Promise((resolve) => {
      const url = `http://localhost:${service.port}${service.healthPath}`;
      const req = http.get(url, { timeout: 5000 }, (res) => {
        resolve(res.statusCode === 200);
      });

      req.on('error', () => resolve(false));
      req.on('timeout', () => {
        req.destroy();
        resolve(false);
      });
    });
  }

  async waitForService(serviceName, timeout = 30000) {
    console.log(`⏳ Waiting for ${serviceName} to be ready...`);
    
    const startTime = Date.now();
    while (Date.now() - startTime < timeout) {
      if (await this.checkServiceHealth(serviceName)) {
        console.log(`✅ ${serviceName} is ready!`);
        return true;
      }
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
    
    console.log(`⚠️  ${serviceName} may not be fully ready yet, continuing...`);
    return false;
  }

  async startService(serviceName) {
    if (!SERVICES[serviceName]) {
      console.error(`❌ Unknown service: ${serviceName}`);
      return false;
    }

    const service = SERVICES[serviceName];
    const servicePath = path.join(this.projectRoot, service.path);

    if (!fs.existsSync(servicePath)) {
      console.error(`❌ Service path not found: ${servicePath}`);
      return false;
    }

    console.log(`🚀 Starting ${serviceName} service...`);

    try {
      const env = { 
        ...process.env, 
        NODE_ENV: 'development',
        DATABASE_URL_NEON: process.env.DATABASE_URL_NEON || ''
      };

      const childProcess = spawn(service.command, service.args, {
        cwd: servicePath,
        env: env,
        stdio: 'inherit',
        shell: process.platform === 'win32'
      });

      this.processes.set(serviceName, childProcess);

      childProcess.on('error', (error) => {
        console.error(`❌ Failed to start ${serviceName}: ${error.message}`);
      });

      childProcess.on('exit', (code) => {
        console.log(`🛑 ${serviceName} exited with code ${code}`);
        this.processes.delete(serviceName);
      });

      // Wait a moment to see if it starts successfully
      await new Promise(resolve => setTimeout(resolve, 2000));

      if (this.processes.has(serviceName)) {
        console.log(`✅ ${serviceName} service started on port ${service.port}`);
        return true;
      } else {
        console.error(`❌ ${serviceName} service failed to start`);
        return false;
      }

    } catch (error) {
      console.error(`❌ Failed to start ${serviceName}: ${error.message}`);
      return false;
    }
  }

  async smartStart() {
    console.log('🚀 ConHub Smart Start');
    console.log('='.repeat(50));

    // Check prerequisites first
    const prereqsOk = await this.checkPrerequisites();
    if (!prereqsOk) {
      console.error('❌ Prerequisites not met, aborting start');
      process.exit(1);
    }

    // Start services in optimal order
    console.log('\n🚀 Starting services in optimal order...');
    
    // 1. Start auth service first (other services depend on it)
    console.log('\n1️⃣  Starting Auth Service...');
    await this.startService('auth');
    await this.waitForService('auth', 30000);

    // 2. Start data service (core backend functionality)
    console.log('\n2️⃣  Starting Data Service...');
    await this.startService('data');
    await this.waitForService('data', 20000);

    // 3. Start other backend services in parallel
    console.log('\n3️⃣  Starting other backend services...');
    const otherServices = ['billing', 'security', 'webhook'];
    await Promise.all(otherServices.map(service => this.startService(service)));
    
    // Wait for all backend services
    for (const service of otherServices) {
      await this.waitForService(service, 15000);
    }

    // 4. Start frontend last
    console.log('\n4️⃣  Starting Frontend...');
    await this.startService('frontend');
    await this.waitForService('frontend', 30000);

    console.log('\n🎉 ConHub startup complete!');
    console.log('🌐 Frontend: http://localhost:3000');
    console.log('🔐 Auth API: http://localhost:3010');
    console.log('📊 Data API: http://localhost:3013');
  }

  async startAll() {
    return this.smartStart();
  }

  async startSpecific(services) {
    console.log(`🚀 Starting services: ${services.join(', ')}`);
    
    for (const service of services) {
      await this.startService(service);
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
}

async function main() {
  const args = process.argv.slice(2);
  const serviceManager = new ServiceManager();

  if (args.length === 0 || args[0] === 'all') {
    await serviceManager.startAll();
  } else {
    await serviceManager.startSpecific(args);
  }
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = ServiceManager;
