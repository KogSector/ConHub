// Test specific GitHub API endpoints that the backend calls
const token = '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';

async function testGitHubEndpoints(owner, repo) {
  console.log(`🔍 Testing all GitHub API endpoints for ${owner}/${repo}\n`);
  
  const endpoints = [
    {
      name: 'Repository Info',
      url: `https://api.github.com/repos/${owner}/${repo}`,
      description: 'Basic repository metadata'
    },
    {
      name: 'Branches List',
      url: `https://api.github.com/repos/${owner}/${repo}/branches`,
      description: 'List of repository branches'
    },
    {
      name: 'Contents Root',
      url: `https://api.github.com/repos/${owner}/${repo}/contents`,
      description: 'Repository root contents'
    },
    {
      name: 'Repository README',
      url: `https://api.github.com/repos/${owner}/${repo}/readme`,
      description: 'Repository README file'
    }
  ];

  for (const endpoint of endpoints) {
    console.log(`📡 Testing: ${endpoint.name}`);
    console.log(`   URL: ${endpoint.url}`);
    console.log(`   Purpose: ${endpoint.description}`);
    
    try {
      const response = await fetch(endpoint.url, {
        headers: {
          'Authorization': `token ${token}`, // Same as backend
          'Accept': 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
          'User-Agent': 'ConHub'
        }
      });

      if (response.ok) {
        const data = await response.json();
        console.log(`   ✅ Status: ${response.status} OK`);
        
        // Show relevant info for each endpoint
        if (endpoint.name === 'Repository Info') {
          console.log(`   📊 Private: ${data.private}, Default Branch: ${data.default_branch}`);
        } else if (endpoint.name === 'Branches List') {
          console.log(`   🌿 Found ${data.length} branches: ${data.slice(0, 3).map(b => b.name).join(', ')}${data.length > 3 ? '...' : ''}`);
        } else if (endpoint.name === 'Contents Root') {
          console.log(`   📁 Found ${data.length} items in root`);
        } else if (endpoint.name === 'Repository README') {
          console.log(`   📄 README found: ${data.name}`);
        }
      } else {
        console.log(`   ❌ Status: ${response.status} ${response.statusText}`);
        const errorText = await response.text();
        console.log(`   📄 Error: ${errorText.substring(0, 200)}${errorText.length > 200 ? '...' : ''}`);
        
        if (response.status === 403) {
          console.log(`   ⚠️  This endpoint might be causing the backend error!`);
        }
      }
    } catch (error) {
      console.log(`   ❌ Network Error: ${error.message}`);
    }
    
    console.log(); // Empty line for readability
  }
}

async function runEndpointTests() {
  console.log('🚀 GitHub API Endpoint Tests');
  console.log('=' .repeat(50));
  console.log('Testing the same endpoints that the backend calls...\n');

  // Test the failing repository
  await testGitHubEndpoints('Skylto-inc', 'DealMate');
  
  console.log('=' .repeat(50));
  console.log('📋 Summary:');
  console.log('   If any endpoint shows 403 errors, that\'s likely the cause');
  console.log('   of the backend "Permission denied" error.');
  console.log('   The backend successfully calls the first endpoint but');
  console.log('   may fail on subsequent endpoints like branches or contents.');
}

runEndpointTests().catch(console.error);