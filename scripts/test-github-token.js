// GitHub Token Test Script
// This script tests the GitHub classic token for repository access

const token = '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';

// Test 1: Check token authentication and scopes
async function testTokenAuth() {
  console.log('🔍 Testing token authentication...');
  
  try {
    const response = await fetch('https://api.github.com/user', {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });

    if (response.ok) {
      const user = await response.json();
      console.log('✅ Token authentication successful!');
      console.log(`   User: ${user.login}`);
      console.log(`   Account Type: ${user.type}`);
      console.log(`   Public Repos: ${user.public_repos}`);
      
      // Check token scopes from headers
      const scopes = response.headers.get('x-oauth-scopes');
      console.log(`   Token Scopes: ${scopes || 'No scopes header'}`);
      
      return user;
    } else {
      console.log('❌ Token authentication failed');
      console.log(`   Status: ${response.status}`);
      console.log(`   Error: ${await response.text()}`);
      return null;
    }
  } catch (error) {
    console.error('❌ Error testing token auth:', error.message);
    return null;
  }
}

// Test 2: List user's repositories (including private)
async function testUserRepos() {
  console.log('\n🔍 Testing repository access...');
  
  try {
    const response = await fetch('https://api.github.com/user/repos?per_page=10&sort=updated', {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });

    if (response.ok) {
      const repos = await response.json();
      console.log(`✅ Found ${repos.length} repositories:`);
      
      repos.forEach(repo => {
        const privacy = repo.private ? '🔒 Private' : '🌍 Public';
        const owner = repo.owner.type === 'Organization' ? '🏢 Org' : '👤 User';
        console.log(`   ${privacy} ${owner} ${repo.full_name}`);
      });
      
      return repos;
    } else {
      console.log('❌ Failed to access repositories');
      console.log(`   Status: ${response.status}`);
      console.log(`   Error: ${await response.text()}`);
      return null;
    }
  } catch (error) {
    console.error('❌ Error testing repo access:', error.message);
    return null;
  }
}

// Test 3: Test specific repository access
async function testSpecificRepo(owner, repo) {
  console.log(`\n🔍 Testing specific repository: ${owner}/${repo}`);
  
  try {
    const response = await fetch(`https://api.github.com/repos/${owner}/${repo}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });

    if (response.ok) {
      const repoData = await response.json();
      console.log('✅ Repository access successful!');
      console.log(`   Name: ${repoData.full_name}`);
      console.log(`   Private: ${repoData.private ? 'Yes' : 'No'}`);
      console.log(`   Owner Type: ${repoData.owner.type}`);
      console.log(`   Permissions: push=${repoData.permissions?.push}, admin=${repoData.permissions?.admin}`);
      console.log(`   Default Branch: ${repoData.default_branch}`);
      
      return repoData;
    } else {
      console.log('❌ Repository access failed');
      console.log(`   Status: ${response.status}`);
      const errorText = await response.text();
      console.log(`   Error: ${errorText}`);
      
      if (response.status === 404) {
        console.log('   💡 This could mean:');
        console.log('      - Repository doesn\'t exist');
        console.log('      - Repository is private and token lacks access');
        console.log('      - Organization requires token approval');
      }
      
      return null;
    }
  } catch (error) {
    console.error('❌ Error testing specific repo:', error.message);
    return null;
  }
}

// Test 4: Check organization membership and token approval
async function testOrgAccess(orgName) {
  console.log(`\n🔍 Testing organization access: ${orgName}`);
  
  try {
    // Test org membership
    const orgResponse = await fetch(`https://api.github.com/orgs/${orgName}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });

    if (orgResponse.ok) {
      const org = await orgResponse.json();
      console.log(`✅ Organization access: ${org.login}`);
      console.log(`   Type: ${org.type}`);
      console.log(`   Public Repos: ${org.public_repos}`);
    } else {
      console.log(`❌ Organization access failed (${orgResponse.status})`);
    }

    // Test user's membership in org
    const memberResponse = await fetch(`https://api.github.com/orgs/${orgName}/members`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });

    if (memberResponse.ok) {
      console.log(`✅ Can access organization members`);
    } else {
      console.log(`❌ Cannot access org members (${memberResponse.status})`);
    }

  } catch (error) {
    console.error('❌ Error testing org access:', error.message);
  }
}

// Main test function
async function runTests() {
  console.log('🚀 Starting GitHub Token Tests\n');
  console.log('='.repeat(50));

  // Test 1: Authentication
  const user = await testTokenAuth();
  if (!user) {
    console.log('\n❌ Token authentication failed. Cannot proceed with other tests.');
    return;
  }

  // Test 2: Repository listing
  const repos = await testUserRepos();

  // Test 3: If we have repos, test access to first private repo found
  if (repos && repos.length > 0) {
    const privateRepo = repos.find(repo => repo.private);
    if (privateRepo) {
      await testSpecificRepo(privateRepo.owner.login, privateRepo.name);
    }

    // If user is part of organizations, test org access
    const orgRepos = repos.filter(repo => repo.owner.type === 'Organization');
    if (orgRepos.length > 0) {
      const firstOrgRepo = orgRepos[0];
      await testOrgAccess(firstOrgRepo.owner.login);
    }
  }

  // Test 4: Test ConHub repo specifically if it exists
  if (user) {
    await testSpecificRepo('KogSector', 'ConHub');
  }

  console.log('\n' + '='.repeat(50));
  console.log('🏁 Tests completed!');
}

// Run the tests
runTests().catch(console.error);