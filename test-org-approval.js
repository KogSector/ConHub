// GitHub Organization Token Approval Test
// This script specifically tests organization approval for personal access tokens

const token = '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';

// Test specific repository access with detailed error reporting
async function testSpecificRepoAccess(owner, repo) {
  console.log(`🔍 Testing repository access: ${owner}/${repo}`);
  
  try {
    const response = await fetch(`https://api.github.com/repos/${owner}/${repo}`, {
      headers: {
        'Authorization': `token ${token}`, // Using same format as backend
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
        'User-Agent': 'ConHub'
      }
    });

    console.log(`   Status: ${response.status}`);
    console.log(`   Status Text: ${response.statusText}`);
    
    // Check response headers for additional info
    const rateLimitRemaining = response.headers.get('x-ratelimit-remaining');
    const rateLimitReset = response.headers.get('x-ratelimit-reset');
    console.log(`   Rate Limit Remaining: ${rateLimitRemaining}`);
    
    if (response.ok) {
      const repoData = await response.json();
      console.log('✅ Repository access successful!');
      console.log(`   Private: ${repoData.private}`);
      console.log(`   Permissions: ${JSON.stringify(repoData.permissions)}`);
      return true;
    } else {
      console.log('❌ Repository access failed');
      const errorData = await response.text();
      console.log(`   Error Response: ${errorData}`);
      
      if (response.status === 403) {
        console.log('\n💡 403 Forbidden - Possible causes:');
        console.log('   1. Organization requires token approval');
        console.log('   2. Token lacks required permissions');
        console.log('   3. Repository access restricted');
        console.log('\n🔗 Check organization settings:');
        console.log(`   https://github.com/organizations/${owner}/settings/personal-access-tokens`);
      }
      
      return false;
    }
  } catch (error) {
    console.error('❌ Error:', error.message);
    return false;
  }
}

// Test organization token approval status
async function testOrganizationTokenApproval(orgName) {
  console.log(`\n🔍 Testing organization token approval: ${orgName}`);
  
  try {
    // Try to access organization data
    const response = await fetch(`https://api.github.com/orgs/${orgName}`, {
      headers: {
        'Authorization': `token ${token}`,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
        'User-Agent': 'ConHub'
      }
    });

    if (response.ok) {
      const orgData = await response.json();
      console.log('✅ Organization access successful');
      console.log(`   Two Factor Auth Required: ${orgData.two_factor_requirement_enabled}`);
      
      // Check if we can access organization members (indication of proper approval)
      const membersResponse = await fetch(`https://api.github.com/orgs/${orgName}/members`, {
        headers: {
          'Authorization': `token ${token}`,
          'Accept': 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
          'User-Agent': 'ConHub'
        }
      });
      
      if (membersResponse.ok) {
        console.log('✅ Can access organization members - token likely approved');
      } else {
        console.log(`❌ Cannot access org members (${membersResponse.status})`);
        if (membersResponse.status === 403) {
          console.log('   This may indicate token needs approval for private org data');
        }
      }
      
    } else {
      console.log(`❌ Organization access failed (${response.status})`);
      const errorText = await response.text();
      console.log(`   Error: ${errorText}`);
    }
  } catch (error) {
    console.error('❌ Error testing org approval:', error.message);
  }
}

// Test both formats (token vs Bearer) for comparison
async function testBothAuthFormats(owner, repo) {
  console.log(`\n🔄 Testing both auth formats for ${owner}/${repo}:`);
  
  // Test 1: token format (what backend uses)
  console.log('\n1️⃣ Testing "token" format (backend style):');
  const tokenResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}`, {
    headers: {
      'Authorization': `token ${token}`,
      'Accept': 'application/vnd.github+json',
      'User-Agent': 'ConHub'
    }
  });
  console.log(`   Status: ${tokenResponse.status} ${tokenResponse.statusText}`);
  
  // Test 2: Bearer format (modern style)
  console.log('\n2️⃣ Testing "Bearer" format (modern style):');
  const bearerResponse = await fetch(`https://api.github.com/repos/${owner}/${repo}`, {
    headers: {
      'Authorization': `Bearer ${token}`,
      'Accept': 'application/vnd.github+json',
      'User-Agent': 'ConHub'
    }
  });
  console.log(`   Status: ${bearerResponse.status} ${bearerResponse.statusText}`);
  
  if (tokenResponse.status !== bearerResponse.status) {
    console.log('⚠️  Different results between auth formats!');
  }
}

// Main test function
async function runOrgApprovalTests() {
  console.log('🚀 GitHub Organization Token Approval Tests\n');
  console.log('='.repeat(60));

  // Test repositories that showed issues
  await testSpecificRepoAccess('Skylto-inc', 'DealMate');
  await testOrganizationTokenApproval('Skylto-inc');
  await testBothAuthFormats('Skylto-inc', 'DealMate');

  console.log('\n' + '='.repeat(60));
  
  // Also test KogSector (which works)
  console.log('\n🔍 Comparing with working repository:');
  await testSpecificRepoAccess('KogSector', 'ConHub');
  await testOrganizationTokenApproval('KogSector');

  console.log('\n' + '='.repeat(60));
  console.log('🏁 Organization approval tests completed!');
  console.log('\n💡 Next steps if 403 errors persist:');
  console.log('   1. Check organization settings for token approval requirements');
  console.log('   2. Approve your token in the organization settings');
  console.log('   3. Verify you have the required permissions in the organization');
}

// Run the tests
runOrgApprovalTests().catch(console.error);