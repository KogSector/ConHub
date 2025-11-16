const redis = require('redis');
require('dotenv').config({ path: './auth/.env' });

async function testRedis() {
  const redisUrl = process.env.REDIS_URL;
  
  if (!redisUrl) {
    console.error('❌ REDIS_URL not found in environment');
    process.exit(1);
  }

  console.log('🔍 Testing Redis connection...');
  console.log(`📍 URL: ${redisUrl.substring(0, 30)}...`);

  try {
    const client = redis.createClient({
      url: redisUrl,
      socket: {
        connectTimeout: 5000,
        reconnectStrategy: false
      }
    });

    client.on('error', (err) => {
      console.error('❌ Redis Client Error:', err.message);
    });

    await client.connect();
    console.log('✅ Successfully connected to Redis!');
    
    // Test a simple command
    await client.set('test_key', 'test_value');
    const value = await client.get('test_key');
    console.log('✅ Redis read/write test passed:', value);
    
    await client.del('test_key');
    await client.quit();
    
    console.log('✅ Redis is working correctly!');
    process.exit(0);
  } catch (error) {
    console.error('❌ Failed to connect to Redis:', error.message);
    console.error('\n📋 Possible issues:');
    console.error('   1. Redis instance might be down or unreachable');
    console.error('   2. Network/firewall blocking the connection');
    console.error('   3. Invalid credentials or URL');
    console.error('   4. Redis instance needs to be restarted');
    process.exit(1);
  }
}

testRedis();
