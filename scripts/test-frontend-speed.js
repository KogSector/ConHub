#!/usr/bin/env node

const { performance } = require('perf_hooks');

async function testFrontendSpeed() {
  console.log('🚀 Testing Frontend Loading Speed...\n');
  
  const tests = [
    { name: 'Homepage', url: 'http://localhost:3000' },
    { name: 'Dashboard', url: 'http://localhost:3000/dashboard' },
  ];
  
  for (const test of tests) {
    console.log(`Testing ${test.name}...`);
    const start = performance.now();
    
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 10000); // 10 second timeout
      
      const response = await fetch(test.url, { 
        signal: controller.signal,
        headers: { 
          'User-Agent': 'ConHub Speed Test',
          'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
        }
      });
      
      clearTimeout(timeoutId);
      const end = performance.now();
      const time = Math.round(end - start);
      
      if (response.ok) {
        const status = time < 1000 ? '🟢 FAST' : time < 3000 ? '🟡 OKAY' : '🔴 SLOW';
        console.log(`  ${status} ${test.name}: ${time}ms (Status: ${response.status})`);
      } else {
        console.log(`  ❌ ${test.name}: HTTP ${response.status} (${time}ms)`);
      }
    } catch (error) {
      const end = performance.now();
      const time = Math.round(end - start);
      
      if (error.name === 'AbortError') {
        console.log(`  ⏰ ${test.name}: TIMEOUT after ${time}ms`);
      } else {
        console.log(`  💥 ${test.name}: ${error.message} (${time}ms)`);
      }
    }
  }
  
  console.log('\n📊 Speed Guidelines:');
  console.log('  🟢 < 1000ms = Fast');
  console.log('  🟡 1000-3000ms = Acceptable');
  console.log('  🔴 > 3000ms = Needs optimization');
}

// Only run if called directly
if (require.main === module) {
  testFrontendSpeed().catch(console.error);
}

module.exports = { testFrontendSpeed };