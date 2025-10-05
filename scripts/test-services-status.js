#!/usr/bin/env node

/**
 * Quick service status checker for ConHub
 */

const axios = require('axios');

const services = [
  { name: 'Backend', url: 'http://localhost:3001/health', port: 3001 },
  { name: 'Frontend', url: 'http://localhost:3000', port: 3000 },
  { name: 'AI Service', url: 'http://localhost:8001/health', port: 8001 },
  { name: 'Lexor', url: 'http://localhost:3002/health', port: 3002 }
];

async function checkService(service) {
  try {
    const response = await axios.get(service.url, { 
      timeout: 3000,
      validateStatus: () => true // Accept any status code
    });
    
    if (response.status < 500) {
      console.log(`✅ ${service.name} (Port ${service.port}): Running (Status: ${response.status})`);
      return true;
    } else {
      console.log(`⚠️  ${service.name} (Port ${service.port}): Server Error (Status: ${response.status})`);
      return false;
    }
  } catch (error) {
    if (error.code === 'ECONNREFUSED') {
      console.log(`❌ ${service.name} (Port ${service.port}): Not Running`);
    } else {
      console.log(`⚠️  ${service.name} (Port ${service.port}): ${error.message}`);
    }
    return false;
  }
}

async function checkAllServices() {
  console.log('🔍 Checking ConHub Services Status...\n');
  
  const results = [];
  for (const service of services) {
    const isRunning = await checkService(service);
    results.push({ ...service, running: isRunning });
  }
  
  console.log('\n📊 Summary:');
  const runningCount = results.filter(s => s.running).length;
  console.log(`${runningCount}/${results.length} services are running`);
  
  if (runningCount === 0) {
    console.log('\n🚀 To start all services, run: npm start');
  } else if (runningCount < results.length) {
    console.log('\n⚠️  Some services are not running. Check the logs or restart with: npm start');
  } else {
    console.log('\n🎉 All services are running! You can now test the AI agents.');
  }
  
  return runningCount === results.length;
}

if (require.main === module) {
  checkAllServices()
    .then(allRunning => {
      process.exit(allRunning ? 0 : 1);
    })
    .catch(error => {
      console.error('Service check failed:', error);
      process.exit(1);
    });
}

module.exports = { checkAllServices };