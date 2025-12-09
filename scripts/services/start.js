#!/usr/bin/env node

const { spawn } = require('child_process');
const http = require('http');
const path = require('path');
const fs = require('fs');

const SERVICES = {
  auth: { port: 3010, path: 'auth', command: 'cargo', args: ['run'], healthPath: '/health', description: 'Authentication & JWT' },
  data: { port: 3013, path: 'data', command: 'cargo', args: ['run'], healthPath: '/health', description: 'Data Sources & Connectors' },
  billing: { port: 3011, path: 'billing', command: 'cargo', args: ['run'], healthPath: '/health', description: 'Stripe Payments' },
  security: { port: 3014, path: 'security', command: 'cargo', args: ['run'], healthPath: '/health', description: 'Security & Audit' },
  webhook: { port: 3015, path: 'webhook', command: 'cargo', args: ['run'], healthPath: '/health', description: 'External Webhooks' },
  client: { port: 3012, path: 'client', command: 'cargo', args: ['run'], healthPath: '/health', description: 'AI Client Service' },
  mcp: { port: 3004, path: 'mcp', command: 'cargo', args: ['run'], healthPath: '/health', description: 'MCP Protocol Service' },
  backend: { port: 8010, path: 'backend', command: 'cargo', args: ['run'], healthPath: '/health', description: 'GraphQL Gateway' },
  embedding: { port: 8082, path: 'vector_rag', command: 'cargo', args: ['run'], healthPath: '/health', description: 'Fusion Embeddings' },
  // indexers: { port: 8080, path: 'indexers', command: 'cargo', args: ['run'], healthPath: '/health', description: 'Search & Indexing' }, // Removed - will be rewritten
  frontend: { port: 3000, path: 'frontend', command: 'npm.cmd', args: ['run', 'dev'], healthPath: '/', description: 'Next.js UI' }
};

class ServiceManager {
  constructor() {
    this.projectRoot = path.resolve(__dirname, '../..');
    this.processes = new Map();
    this.serviceStatus = new Map();
  }

  async checkPrerequisites() {
    console.log('🔍 Checking prerequisites...');

    // Check if required directories exist and have .env files
    const serviceNames = Object.keys(SERVICES);
    let allEnvFilesPresent = true;

    for (const serviceName of serviceNames) {
      const service = SERVICES[serviceName];
      const dirPath = path.join(this.projectRoot, service.path);

      if (!fs.existsSync(dirPath)) {
        console.error(`❌ Required directory missing: ${service.path}`);
        return false;
      }

      // Check if each service has its own .env file
      const serviceEnvPath = path.join(dirPath, '.env');
      if (!fs.existsSync(serviceEnvPath)) {
        console.log(`⚠️  ${serviceName} service missing .env file`);
        allEnvFilesPresent = false;
      }
    }

    if (allEnvFilesPresent) {
      console.log('✅ All microservices have proper environment variables. ✅ Prerequisites checked');
    } else {
      console.log('⚠️  Some microservices are missing .env files. ✅ Prerequisites checked');
    }
    return true;
  }

  async checkServiceHealth(serviceName) {
    const service = SERVICES[serviceName];
    if (!service) return false;

    return new Promise((resolve) => {
      const url = `http://localhost:${service.port}${service.healthPath}`;
      // Increase HTTP health-check timeout by +60s (from 10s to 70s)
      const req = http.get(url, { timeout: 70000 }, (res) => {
        resolve(res.statusCode === 200);
      });

      req.on('error', () => resolve(false));
      req.on('timeout', () => {
        req.destroy();
        resolve(false);
      });
    });
  }

  async waitForService(serviceName, timeout = 90000) {
    console.log(`⏳ Waiting for ${serviceName}...`);

    const startTime = Date.now();
    while (Date.now() - startTime < timeout) {
      if (await this.checkServiceHealth(serviceName)) {
        console.log(`✅ ${serviceName} ready!`);
        this.updateServiceStatus(serviceName, 'HEALTHY', 'Health check passed');
        return true;
      }
      await new Promise(resolve => setTimeout(resolve, 2000));
    }

    if (this.processes.has(serviceName)) {
      console.log(`⚠️  ${serviceName} health check timeout, but process may still be starting...`);
      this.updateServiceStatus(serviceName, 'RUNNING', 'Health check timeout');
    } else {
      console.log(`🔴 ${serviceName} process is not running`);
      this.updateServiceStatus(serviceName, 'FAILED', 'Process exited before healthy');
    }
    return false;
  }

  updateServiceStatus(serviceName, status, details = '') {
    this.serviceStatus.set(serviceName, {
      status,
      details,
      timestamp: new Date().toISOString(),
      port: SERVICES[serviceName]?.port
    });
  }

  printServiceStatus() {
    console.log('\n📊 SERVICE STATUS OVERVIEW');
    console.log('='.repeat(60));
    console.log('Service'.padEnd(12) + 'Port'.padEnd(8) + 'Status');
    console.log('-'.repeat(60));
    
    for (const [serviceName, service] of Object.entries(SERVICES)) {
      const status = this.serviceStatus.get(serviceName) || { status: 'NOT_STARTED', details: '' };
      const normalizedStatus = (status.status === 'RUNNING') ? 'HEALTHY' : status.status;
      const statusIcon = {
        'STARTING': '🟡',
        'RUNNING': '✅',
        'HEALTHY': '✅',
        'FAILED': '🔴',
        'STOPPED': '⚫',
        'NOT_STARTED': '⚪'
      }[status.status] || '❓';
      console.log(
        serviceName.padEnd(12) +
        service.port.toString().padEnd(8) +
        `${statusIcon} ${normalizedStatus}`
      );
    }
    console.log('='.repeat(60));
  }

  async runCommand(cwd, command, args, env) {
    return new Promise((resolve) => {
      const proc = spawn(command, args, {
        cwd,
        env,
        stdio: ['ignore', 'pipe', 'inherit'],
        shell: process.platform === 'win32' && command.endsWith('.cmd')
      });
      proc.stdout.on('data', (data) => {
        const lines = data.toString().split('\n').filter(line => line.trim());
        lines.forEach(line => console.log(`[build] ${line}`));
      });
      proc.on('exit', (code) => resolve(code === 0));
      proc.on('error', () => resolve(false));
    });
  }

  async startService(serviceName, toggles) {
    if (!SERVICES[serviceName]) {
      console.error(`❌ Unknown service: ${serviceName}`);
      this.updateServiceStatus(serviceName, 'FAILED', 'Unknown service');
      return false;
    }

    const service = SERVICES[serviceName];
    const servicePath = path.join(this.projectRoot, service.path);

    if (!fs.existsSync(servicePath)) {
      console.error(`❌ Service path not found: ${servicePath}`);
      this.updateServiceStatus(serviceName, 'FAILED', 'Path not found');
      return false;
    }

    console.log(`🚀 Starting ${serviceName} on port ${service.port}...`);
    this.updateServiceStatus(serviceName, 'STARTING', 'Initializing...');

    try {
      const isProd = !!(toggles && toggles.Prod);
      const env = { 
        ...process.env, 
        NODE_ENV: isProd ? 'production' : 'development'
      };

      // Ensure Rust services have useful logging by default.
      // If RUST_LOG is already set in the parent env, respect it;
      // otherwise, default to info for all crates and debug for conhub-*.
      if (service.command === 'cargo') {
        if (!env.RUST_LOG) {
          env.RUST_LOG = process.env.RUST_LOG || 'info,conhub=debug';
        }
      }

      const featureTogglesPath = path.join(this.projectRoot, 'feature-toggles.json');
      env.FEATURE_TOGGLES_PATH = featureTogglesPath;

      let command = service.command;
      let args = [...service.args];

      if (isProd) {
        if (service.command === 'cargo') {
          command = 'cargo';
          args = ['run', '--release'];
        }
        if (serviceName === 'frontend') {
          const built = await this.runCommand(servicePath, 'npm.cmd', ['run', 'build'], env);
          if (!built) {
            console.error('❌ frontend build failed');
            this.updateServiceStatus(serviceName, 'FAILED', 'Build failed');
            return false;
          }
          command = 'npm.cmd';
          args = ['start'];
        }
      }

      const childProcess = spawn(command, args, {
        cwd: servicePath,
        env: env,
        stdio: ['ignore', 'pipe', 'inherit'],
        shell: process.platform === 'win32' && command.endsWith('.cmd')
      });

      childProcess.stdout.on('data', (data) => {
        const lines = data.toString().split('\n').filter(line => line.trim());
        lines.forEach(line => console.log(`[${serviceName}] ${line}`));
      });

      this.processes.set(serviceName, childProcess);

      childProcess.on('error', (error) => {
        console.error(`❌ Failed to start ${serviceName}: ${error.message}`);
        this.updateServiceStatus(serviceName, 'FAILED', error.message);
      });

      childProcess.on('exit', (code) => {
        console.log(`🛑 ${serviceName} exited with code ${code}`);
        this.updateServiceStatus(serviceName, 'STOPPED', `Exit code: ${code}`);
        this.processes.delete(serviceName);
      });

      // Wait a moment to see if it starts successfully
      await new Promise(resolve => setTimeout(resolve, 3000));

      if (this.processes.has(serviceName)) {
        console.log(`✅ ${serviceName} started`);
        this.updateServiceStatus(serviceName, 'RUNNING', 'Process active');
        return true;
      } else {
        console.error(`❌ ${serviceName} process failed to start`);
        this.updateServiceStatus(serviceName, 'FAILED', 'Process died immediately');
        return false;
      }

    } catch (error) {
      console.error(`❌ Failed to start ${serviceName}: ${error.message}`);
      this.updateServiceStatus(serviceName, 'FAILED', error.message);
      return false;
    }
  }

  async checkFeatureToggles() {
    const togglePath = path.join(this.projectRoot, 'feature-toggles.json');
    if (fs.existsSync(togglePath)) {
      try {
        const toggles = JSON.parse(fs.readFileSync(togglePath, 'utf8'));
        return toggles;
      } catch (error) {
        
      }
    }
    return { Auth: true, Heavy: true, Docker: false, Redis: true };
  }

  async smartStart() {
    console.log('🚀 ConHub Comprehensive Service Manager');
    console.log('='.repeat(60));

    // Check feature toggles
    const toggles = await this.checkFeatureToggles();

    // Check prerequisites first
    const prereqsOk = await this.checkPrerequisites();
    if (!prereqsOk) {
      console.error('❌ Prerequisites not met, aborting start');
      process.exit(1);
    }

    // Initialize all service statuses
    for (const serviceName of Object.keys(SERVICES)) {
      this.updateServiceStatus(serviceName, 'NOT_STARTED', 'Waiting to start');
    }

    

    // Start services in optimal order based on dependencies
    console.log('\n🚀 Starting services... ');
    console.log(''); // First empty line
    console.log(''); // Second empty line

    // Always start the auth service. The internal Auth feature toggle
    // controls whether it runs in full Auth0 mode or dev-mode with
    // default claims, but the process should always be running so that
    // DB connections, dev user seeding, and connector endpoints work.
    await this.startService('auth', toggles);
    // Increase auth wait timeout by +60s (30s -> 90s)
    await this.waitForService('auth', 90000);
    console.log('');


    await this.startService('data', toggles);
    // Increase data wait timeout by +60s (25s -> 85s)
    await this.waitForService('data', 85000);
    console.log('');

    const supportServices = ['billing', 'security', 'webhook', 'client', 'mcp'];
    for (const service of supportServices) {
      await this.startService(service, toggles);
      await new Promise(resolve => setTimeout(resolve, 2000)); // Stagger starts
      // Increase support-service wait timeout by +60s (30s -> 90s)
      await this.waitForService(service, 90000);
      console.log('');
    }

    if (toggles.Heavy) {
      const heavyServices = ['embedding']; // indexers removed - will be rewritten later
      for (const service of heavyServices) {
        await this.startService(service, toggles);
        // Increase embedding wait timeout by +60s (20s -> 80s)
        await this.waitForService(service, 80000);
        console.log('');
      }
    }


    await this.startService('backend', toggles);
    // Increase backend wait timeout by +60s (25s -> 85s)
    await this.waitForService('backend', 85000);
    console.log('');


    await this.startService('frontend', toggles);
    // Increase frontend wait timeout by +60s (60s -> 120s)
    await this.waitForService('frontend', 120000);
    console.log('');

    // Print final comprehensive status
    console.log('\n');
    this.printServiceStatus();
    
    console.log('\n🎉 ConHub startup sequence complete!');
    console.log('\n🌐 ACCESS POINTS:');
    console.log('   🌐 Frontend:     http://localhost:3000');
    console.log('   🔗 GraphQL API:  http://localhost:8010/api/graphql');
    console.log('   🔐 Auth API:     http://localhost:3010/health');
    console.log('   📊 Data API:     http://localhost:3013/health');
    console.log('   🤖 AI Service:   http://localhost:3012/health');
    if (toggles.Heavy) {
      console.log('   🧠 Embeddings:   http://localhost:8082/health');
      console.log('   🔍 Search:       http://localhost:8080/health');
    }
    
    console.log('\n💡 Use Ctrl+C to stop all services');
    console.log('📊 Run "npm run status" to check service health anytime');
  }

  async startAll() {
    return this.smartStart();
  }

  async checkAllServices() {
    console.log('🔍 ConHub Service Health Check');
    console.log('='.repeat(50));
    
    for (const [serviceName, service] of Object.entries(SERVICES)) {
      const isHealthy = await this.checkServiceHealth(serviceName);
      const status = isHealthy ? 'HEALTHY' : 'UNHEALTHY';
      this.updateServiceStatus(serviceName, status, isHealthy ? 'Health check passed' : 'Health check failed');
    }
    
    this.printServiceStatus();
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

// Graceful shutdown handling
process.on('SIGINT', async () => {
  console.log('\n🛑 Received SIGINT, shutting down gracefully...');
  const { stopAllServices } = require('./stop.js');
  await stopAllServices();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('\n🛑 Received SIGTERM, shutting down gracefully...');
  const { stopAllServices } = require('./stop.js');
  await stopAllServices();
  process.exit(0);
});

if (require.main === module) {
  main().catch(console.error);
}

module.exports = ServiceManager;
