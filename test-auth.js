// Test script for authentication system
const API_BASE = 'http://localhost:3001';

async function testAuth() {
  console.log('🔐 Testing ConHub Authentication System\n');

  try {
    // Test 1: Register a new user
    console.log('1. Testing user registration...');
    const registerResponse = await fetch(`${API_BASE}/api/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: 'test@conhub.dev',
        password: 'testpassword123',
        name: 'Test User',
        organization: 'ConHub Testing'
      })
    });

    if (registerResponse.ok) {
      const registerData = await registerResponse.json();
      console.log('✅ Registration successful');
      console.log(`   User: ${registerData.user.name} (${registerData.user.email})`);
      console.log(`   Token: ${registerData.token.substring(0, 20)}...`);
    } else {
      console.log('❌ Registration failed:', await registerResponse.text());
    }

    // Test 2: Login with mock admin user
    console.log('\n2. Testing admin login...');
    const loginResponse = await fetch(`${API_BASE}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: 'admin@conhub.dev',
        password: 'password123'
      })
    });

    if (loginResponse.ok) {
      const loginData = await loginResponse.json();
      console.log('✅ Login successful');
      console.log(`   User: ${loginData.user.name} (${loginData.user.email})`);
      console.log(`   Role: ${loginData.user.role}`);
      console.log(`   Subscription: ${loginData.user.subscription_tier}`);
      console.log(`   Token: ${loginData.token.substring(0, 20)}...`);
      
      // Test 3: Verify token
      console.log('\n3. Testing token verification...');
      const verifyResponse = await fetch(`${API_BASE}/api/auth/verify`, {
        method: 'POST',
        headers: { 
          'Authorization': `Bearer ${loginData.token}`,
          'Content-Type': 'application/json' 
        }
      });

      if (verifyResponse.ok) {
        const verifyData = await verifyResponse.json();
        console.log('✅ Token verification successful');
        console.log(`   Valid: ${verifyData.valid}`);
      } else {
        console.log('❌ Token verification failed');
      }

      // Test 4: Get user profile
      console.log('\n4. Testing profile retrieval...');
      const profileResponse = await fetch(`${API_BASE}/api/auth/profile`, {
        headers: { 
          'Authorization': `Bearer ${loginData.token}`,
          'Content-Type': 'application/json' 
        }
      });

      if (profileResponse.ok) {
        const profileData = await profileResponse.json();
        console.log('✅ Profile retrieval successful');
        console.log(`   Profile: ${JSON.stringify(profileData, null, 2)}`);
      } else {
        console.log('❌ Profile retrieval failed');
      }

    } else {
      console.log('❌ Login failed:', await loginResponse.text());
    }

    // Test 5: Invalid login
    console.log('\n5. Testing invalid login...');
    const invalidLoginResponse = await fetch(`${API_BASE}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: 'invalid@example.com',
        password: 'wrongpassword'
      })
    });

    if (invalidLoginResponse.status === 401) {
      console.log('✅ Invalid login correctly rejected');
    } else {
      console.log('❌ Invalid login should have been rejected');
    }

  } catch (error) {
    console.error('❌ Test failed with error:', error.message);
  }

  console.log('\n🏁 Authentication tests completed');
}

// Run the tests
testAuth();