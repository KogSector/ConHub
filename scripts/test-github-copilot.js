#!/usr/bin/env node

const axios = require('axios');
const fs = require('fs');
const path = require('path');

// Configuration
const API_BASE_URL = process.env.LANGCHAIN_SERVICE_URL || 'http://localhost:3001';
const GITHUB_TOKEN = process.env.GITHUB_TOKEN;

// Test results
const testResults = {
  passed: 0,
  failed: 0,
  tests: []
};

// Test helper function
async function runTest(name, testFn) {
  console.log(`🧪 Running test: ${name}`);
  try {
    await testFn();
    testResults.passed++;
    testResults.tests.push({ name, status: 'PASSED' });
    console.log(`✅ ${name} - PASSED`);
  } catch (error) {
    testResults.failed++;
    testResults.tests.push({ name, status: 'FAILED', error: error.message });
    console.log(`❌ ${name} - FAILED: ${error.message}`);
  }
}

// API helper function
async function apiCall(endpoint, options = {}) {
  const config = {
    url: `${API_BASE_URL}${endpoint}`,
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      ...(GITHUB_TOKEN && { 'Authorization': `Bearer ${GITHUB_TOKEN}` })
    },
    ...options
  };

  const response = await axios(config);
  return response.data;
}

// Test service health
async function testServiceHealth() {
  const response = await apiCall('/health');
  if (response.status !== 'OK') {
    throw new Error('Service health check failed');
  }
}

// Test GitHub authentication (if token provided)
async function testGitHubAuth() {
  if (!GITHUB_TOKEN) {
    console.log('⚠️  Skipping GitHub auth test - no token provided');
    return;
  }
  
  const response = await apiCall('/api/github/user');
  if (!response.success || !response.data.login) {
    throw new Error('GitHub authentication failed');
  }
  console.log(`📱 Authenticated as: ${response.data.login}`);
}

// Test GitHub repositories endpoint
async function testGitHubRepositories() {
  if (!GITHUB_TOKEN) {
    console.log('⚠️  Skipping GitHub repositories test - no token provided');
    return;
  }
  
  const response = await apiCall('/api/github/repositories?per_page=5');
  if (!response.success || !Array.isArray(response.data)) {
    throw new Error('Failed to fetch repositories');
  }
  console.log(`📚 Found ${response.data.length} repositories`);
}

// Test GitHub organizations endpoint
async function testGitHubOrganizations() {
  if (!GITHUB_TOKEN) {
    console.log('⚠️  Skipping GitHub organizations test - no token provided');
    return;
  }
  
  const response = await apiCall('/api/github/organizations');
  if (!response.success || !Array.isArray(response.data)) {
    throw new Error('Failed to fetch organizations');
  }
  console.log(`🏢 Found ${response.data.length} organizations`);
}

// Test GitHub search
async function testGitHubSearch() {
  if (!GITHUB_TOKEN) {
    console.log('⚠️  Skipping GitHub search test - no token provided');
    return;
  }
  
  const response = await apiCall('/api/github/search/repositories?q=javascript&per_page=5');
  if (!response.success || !response.data.items) {
    throw new Error('Failed to search repositories');
  }
  console.log(`🔍 Search found ${response.data.total_count} repositories`);
}

// Test Copilot user info (optional)
async function testCopilotUserInfo() {
  if (!GITHUB_TOKEN) {
    console.log('⚠️  Skipping Copilot test - no token provided');
    return;
  }
  
  try {
    const response = await apiCall('/api/copilot/user/info');
    if (response.success) {
      console.log(`🤖 Copilot status: Available`);
    }
  } catch (error) {
    // Copilot might not be available for this user
    console.log('ℹ️  Copilot not available for this user');
  }
}

// Test file structure
async function testFileStructure() {
  const requiredFiles = [
    'langchain-service/src/services/github/copilotService.ts',
    'langchain-service/src/services/github/integrationService.ts',
    'langchain-service/src/services/auth/githubAppAuth.ts',
    'langchain-service/src/routes/github.ts',
    'langchain-service/src/routes/copilot.ts',
    'frontend/components/github/copilot-dashboard.tsx',
    'frontend/app/github-copilot/page.tsx',
    'mcp-github-copilot.json'
  ];

  for (const file of requiredFiles) {
    const filePath = path.join(__dirname, '..', file);
    if (!fs.existsSync(filePath)) {
      throw new Error(`Required file missing: ${file}`);
    }
  }
  console.log(`📁 All ${requiredFiles.length} required files exist`);
}

// Test TypeScript compilation
async function testTypeScriptCompilation() {
  const { exec } = require('child_process');
  const { promisify } = require('util');
  const execAsync = promisify(exec);

  try {
    await execAsync('cd langchain-service && npm run build');
    console.log('🔨 TypeScript compilation successful');
  } catch (error) {
    throw new Error(`TypeScript compilation failed: ${error.message}`);
  }
}

// Main test runner
async function runAllTests() {
  console.log('🚀 Starting GitHub + Copilot Integration Tests\n');
  
  // File structure tests
  await runTest('File Structure Check', testFileStructure);
  await runTest('TypeScript Compilation', testTypeScriptCompilation);
  
  // Service tests
  await runTest('Service Health Check', testServiceHealth);
  
  // GitHub API tests (require token)
  await runTest('GitHub Authentication', testGitHubAuth);
  await runTest('GitHub Repositories', testGitHubRepositories);
  await runTest('GitHub Organizations', testGitHubOrganizations);
  await runTest('GitHub Search', testGitHubSearch);
  await runTest('Copilot User Info', testCopilotUserInfo);
  
  // Print summary
  console.log('\n📊 Test Summary:');
  console.log(`✅ Passed: ${testResults.passed}`);
  console.log(`❌ Failed: ${testResults.failed}`);
  console.log(`📈 Success Rate: ${((testResults.passed / (testResults.passed + testResults.failed)) * 100).toFixed(1)}%`);
  
  if (testResults.failed > 0) {
    console.log('\n❌ Failed Tests:');
    testResults.tests
      .filter(test => test.status === 'FAILED')
      .forEach(test => console.log(`  • ${test.name}: ${test.error}`));
  }
  
  console.log('\n📝 Notes:');
  console.log('• GitHub API tests require GITHUB_TOKEN environment variable');
  console.log('• Copilot tests require appropriate GitHub permissions');
  console.log('• Some tests may be skipped if dependencies are not available');
  
  // Exit with appropriate code
  process.exit(testResults.failed > 0 ? 1 : 0);
}

// Run tests if this script is executed directly
if (require.main === module) {
  runAllTests().catch(error => {
    console.error('❌ Test runner failed:', error.message);
    process.exit(1);
  });
}

module.exports = { runAllTests, testResults };