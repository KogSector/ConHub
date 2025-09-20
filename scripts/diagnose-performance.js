const { performance } = require('perf_hooks');

async function checkService(name, url, timeout = 5000) {
  const start = performance.now();
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);
    
    const response = await fetch(url, { 
      signal: controller.signal,
      headers: { 'Accept': 'application/json' }
    });
    
    clearTimeout(timeoutId);
    const end = performance.now();
    const time = Math.round(end - start);
    
    if (response.ok) {
      console.log(`✅ ${name}: ${time}ms (${response.status})`);
      return { success: true, time, status: response.status };
    } else {
      console.log(`❌ ${name}: ${response.status} (${time}ms)`);
      return { success: false, time, status: response.status };
    }
  } catch (error) {
    const end = performance.now();
    const time = Math.round(end - start);
    
    if (error.name === 'AbortError') {
      console.log(`⏰ ${name}: Timeout after ${time}ms`);
    } else {
      console.log(`💥 ${name}: ${error.message} (${time}ms)`);
    }
    return { success: false, time, error: error.message };
  }
}

async function diagnosePerformance() {
  console.log('🔍 ConHub Performance Diagnosis');
  console.log('================================');
  
  const services = [
    { name: 'Frontend', url: 'http://localhost:3000' },
    { name: 'Backend', url: 'http://localhost:3001' },
    { name: 'LangChain', url: 'http://localhost:3002' },
    { name: 'Haystack', url: 'http://localhost:8001' }
  ];
  
  const results = [];
  
  for (const service of services) {
    console.log(`\n🔄 Checking ${service.name}...`);
    const result = await checkService(service.name, service.url);
    results.push({ ...service, ...result });
  }
  
  console.log('\n📊 Summary:');
  console.log('===========');
  
  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);
  
  console.log(`✅ Services running: ${successful.length}/${results.length}`);
  console.log(`❌ Services down: ${failed.length}/${results.length}`);
  
  if (successful.length > 0) {
    const avgTime = Math.round(successful.reduce((sum, r) => sum + r.time, 0) / successful.length);
    console.log(`⚡ Average response time: ${avgTime}ms`);
  }
  
  if (failed.length > 0) {
    console.log('\n🚨 Failed Services:');
    failed.forEach(service => {
      console.log(`   • ${service.name}: ${service.error || 'HTTP ' + service.status}`);
    });
  }
  
  console.log('\n💡 Recommendations:');
  if (failed.some(s => s.name === 'Frontend')) {
    console.log('   • Frontend issue detected - check Next.js compilation');
  }
  if (failed.some(s => s.name === 'Backend')) {
    console.log('   • Backend issue detected - check Rust compilation and database connection');
  }
  if (successful.some(s => s.time > 2000)) {
    console.log('   • Slow response times detected - consider optimizing service startup');
  }
  
  return results;
}

// Export for use in other scripts
module.exports = { diagnosePerformance, checkService };

// Run if called directly
if (require.main === module) {
  diagnosePerformance().catch(console.error);
}