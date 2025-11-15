#!/usr/bin/env node

const ServiceManager = require('./start.js');

async function main() {
  const serviceManager = new ServiceManager();
  await serviceManager.checkAllServices();
}

if (require.main === module) {
  main().catch(console.error);
}