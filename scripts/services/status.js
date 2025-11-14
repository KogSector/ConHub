#!/usr/bin/env node

const http = require('http');
const https = require('https');

const SERVICES = {
  auth: { port: 3010, name: 'Auth Service', healthPath: '/health' },
  data: { port: 3013, name: 'Data Service', healthPath: '/health' },
  billing: { port: 3011, name: 'Billing Service', healthPath: '/health' },
  security: { port: 3014, name: 'Security Service', healthPath: '/health' },
  webhook: { port: 3015, name: 'Webhook Service', healthPath: '/health' },
  frontend: { port: 3000, name: 'Frontend', healthPath: '/' }
};

class StatusChecker {
  async checkService(serviceName, config) {
    return new Promise((resolve) => {
      const url = `http://localhost:${config.port}${config.healthPath}`;
      const client = url.startsWith('https') ? https : http;
      
      const req = client.get(url, { timeout: 5000 }, (res) => {
        resolve({
          name: config.name,
          port: config.port,
          status: res.statusCode === 200 ? '🟢 Running' : `🟡 Issues (${res.statusCode})`,
          url: `localhost:${config.port}`
        });
      });

      req.on('error', () => {
        resolve({
          name: config.name,
          port: config.port,
          status: '🔴 Stopped',
          url: `localhost:${config.port}`
        });
      });

      req.on('timeout', () => {
        req.destroy();
        resolve({
          name: config.name,
          port: config.port,
          status: '🟡 Timeout',
          url: `localhost:${config.port}`
        });
      });
    });
  }

  async checkAll() {
    console.log('📊 ConHub Services Status:');
    console.log('─'.repeat(50));

    const checks = Object.entries(SERVICES).map(([key, config]) => 
      this.checkService(key, config)
    );

    const results = await Promise.all(checks);

    // Sort by port for consistent display
    results.sort((a, b) => a.port - b.port);

    for (const result of results) {
      console.log(`${result.name.padEnd(15)} (${result.url.padEnd(15)}) ${result.status}`);
    }

    console.log('─'.repeat(50));

    // Summary
    const running = results.filter(r => r.status.includes('🟢')).length;
    const total = results.length;
    
    if (running === total) {
      console.log('🎉 All services are running!');
    } else if (running === 0) {
      console.log('🛑 No services are running');
    } else {
      console.log(`⚠️  ${running}/${total} services are running`);
    }

    return results;
  }

  async checkSpecific(services) {
    console.log(`📊 Checking services: ${services.join(', ')}`);
    console.log('─'.repeat(50));

    const checks = services
      .filter(service => SERVICES[service])
      .map(service => this.checkService(service, SERVICES[service]));

    const results = await Promise.all(checks);

    for (const result of results) {
      console.log(`${result.name.padEnd(15)} (${result.url.padEnd(15)}) ${result.status}`);
    }

    return results;
  }
}

async function main() {
  const args = process.argv.slice(2);
  const statusChecker = new StatusChecker();

  if (args.length === 0 || args[0] === 'all') {
    await statusChecker.checkAll();
  } else {
    await statusChecker.checkSpecific(args);
  }
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = StatusChecker;
