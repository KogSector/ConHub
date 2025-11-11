#!/usr/bin/env node

const http = require('http');
const https = require('https');

const services = [
  { name: 'Backend Service', url: 'http://localhost:8000/health' },
  { name: 'Data Service', url: 'http://localhost:3013/health' },
  { name: 'Auth Service', url: 'http://localhost:3010/health' },
  { name: 'Frontend', url: 'http://localhost:3000' }
];

async function checkService(service) {
  return new Promise((resolve) => {
    const client = service.url.startsWith('https') ? https : http;
    const req = client.get(service.url, (res) => {
      resolve({
        name: service.name,
        status: res.statusCode === 200 ? 'UP' : `DOWN (${res.statusCode})`,
        url: service.url
      });
    });

    req.on('error', (err) => {
      resolve({
        name: service.name,
        status: `DOWN (${err.code})`,
        url: service.url
      });
    });

    req.setTimeout(5000, () => {
      req.destroy();
      resolve({
        name: service.name,
        status: 'DOWN (TIMEOUT)',
        url: service.url
      });
    });
  });
}

async function main() {
  console.log('🔍 Checking ConHub services...\n');
  
  const results = await Promise.all(services.map(checkService));
  
  results.forEach(result => {
    const status = result.status === 'UP' ? '✅' : '❌';
    console.log(`${status} ${result.name.padEnd(20)} ${result.status.padEnd(15)} ${result.url}`);
  });

  const allUp = results.every(r => r.status === 'UP');
  
  console.log('\n' + '='.repeat(60));
  console.log(allUp ? '✅ All services are running!' : '❌ Some services are down');
  
  if (!allUp) {
    console.log('\n💡 To start services:');
    console.log('   npm run dev        # Start all services');
    console.log('   npm run dev:data    # Start data service only');
    console.log('   npm run dev:auth    # Start auth service only');
  }
}

main().catch(console.error);