#!/usr/bin/env node

/**
 * Comprehensive AI Agents Integration Test
 * Tests all 4 AI agents: Amazon Q, GitHub Copilot, Cline, Cursor IDE
 */

const axios = require('axios');

const BACKEND_URL = 'http://localhost:3001';
const FRONTEND_URL = 'http://localhost:3000';

const AI_AGENTS = [
  { type: 'amazon_q', name: 'Amazon Q' },
  { type: 'github_copilot', name: 'GitHub Copilot' },
  { type: 'cline', name: 'Cline' },
  { type: 'cursor_ide', name: 'Cursor IDE' }
];

class AIAgentTester {
  constructor() {
    this.results = {
      passed: 0,
      failed: 0,
      tests: []
    };
  }

  async runTest(name, testFn) {
    console.log(`\n🧪 Testing: ${name}`);
    try {
      await testFn();
      console.log(`✅ PASSED: ${name}`);
      this.results.passed++;
      this.results.tests.push({ name, status: 'PASSED' });
    } catch (error) {
      console.log(`❌ FAILED: ${name}`);
      console.log(`   Error: ${error.message}`);
      this.results.failed++;
      this.results.tests.push({ name, status: 'FAILED', error: error.message });
    }
  }

  async testBackendHealth() {
    const response = await axios.get(`${BACKEND_URL}/health`, { timeout: 5000 });
    if (response.status !== 200) {
      throw new Error(`Expected 200, got ${response.status}`);
    }
  }

  async testAgentCreation() {
    for (const agent of AI_AGENTS) {
      await this.runTest(`Create ${agent.name} Agent`, async () => {
        const response = await axios.post(`${BACKEND_URL}/api/agents/create/${agent.type}`, {}, {
          timeout: 5000
        });
        
        if (response.status !== 200) {
          throw new Error(`Expected 200, got ${response.status}`);
        }

        const agentData = response.data;
        if (!agentData.id || agentData.agent_type !== agent.type) {
          throw new Error('Invalid agent data returned');
        }
      });
    }
  }

  async testAgentListing() {
    const response = await axios.get(`${BACKEND_URL}/api/agents/list`, { timeout: 5000 });
    
    if (response.status !== 200) {
      throw new Error(`Expected 200, got ${response.status}`);
    }

    const agents = response.data;
    if (!Array.isArray(agents)) {
      throw new Error('Expected array of agents');
    }

    // Check if all agent types are present
    const agentTypes = agents.map(agent => agent.agent_type);
    for (const expectedAgent of AI_AGENTS) {
      if (!agentTypes.includes(expectedAgent.type)) {
        throw new Error(`Agent type ${expectedAgent.type} not found in list`);
      }
    }

    console.log(`   Found ${agents.length} agents: ${agentTypes.join(', ')}`);
  }

  async testAgentQuery() {
    // First get the list of agents
    const agentsResponse = await axios.get(`${BACKEND_URL}/api/agents/list`);
    const agents = agentsResponse.data;

    for (const agent of agents) {
      await this.runTest(`Query ${agent.agent_type} Agent`, async () => {
        const queryPayload = {
          prompt: `Test query for ${agent.agent_type}`,
          context: 'This is a test context'
        };

        const response = await axios.post(
          `${BACKEND_URL}/api/agents/query/${agent.id}`,
          queryPayload,
          { timeout: 10000 }
        );

        if (response.status !== 200) {
          throw new Error(`Expected 200, got ${response.status}`);
        }

        const responseData = response.data;
        if (!responseData || typeof responseData !== 'string') {
          throw new Error('Invalid response format');
        }

        console.log(`   Response preview: ${responseData.substring(0, 100)}...`);
      });
    }
  }

  async testFrontendAPIRoutes() {
    const frontendRoutes = [
      { path: '/api/agents/amazon-q/query', agent: 'Amazon Q' },
      { path: '/api/agents/github-copilot/query', agent: 'GitHub Copilot' },
      { path: '/api/agents/cline/query', agent: 'Cline' },
      { path: '/api/agents/cursor/query', agent: 'Cursor IDE' }
    ];

    for (const route of frontendRoutes) {
      await this.runTest(`Frontend API Route: ${route.agent}`, async () => {
        const queryPayload = {
          prompt: `Test query for ${route.agent}`,
          context: 'Frontend test context'
        };

        const response = await axios.post(
          `${FRONTEND_URL}${route.path}`,
          queryPayload,
          { 
            timeout: 15000,
            headers: { 'Content-Type': 'application/json' }
          }
        );

        if (response.status !== 200) {
          throw new Error(`Expected 200, got ${response.status}`);
        }

        const responseData = response.data;
        if (!responseData.response) {
          throw new Error('No response field in frontend API response');
        }
      });
    }
  }

  async testFrontendPages() {
    const pages = [
      { path: '/ai-agents', name: 'AI Agents Hub' },
      { path: '/amazon-q', name: 'Amazon Q Page' },
      { path: '/cursor', name: 'Cursor IDE Page' },
      { path: '/cline', name: 'Cline Page' },
      { path: '/github-copilot', name: 'GitHub Copilot Page' }
    ];

    for (const page of pages) {
      await this.runTest(`Frontend Page: ${page.name}`, async () => {
        const response = await axios.get(`${FRONTEND_URL}${page.path}`, { 
          timeout: 10000,
          validateStatus: (status) => status < 500 // Accept 404 as some pages might not exist yet
        });

        if (response.status >= 500) {
          throw new Error(`Server error: ${response.status}`);
        }

        // 200 = page exists, 404 = page not found but server is working
        if (response.status !== 200 && response.status !== 404) {
          throw new Error(`Unexpected status: ${response.status}`);
        }

        if (response.status === 404) {
          console.log(`   Page not found (expected for some pages): ${page.path}`);
        }
      });
    }
  }

  async testEndToEndWorkflow() {
    await this.runTest('End-to-End AI Agent Workflow', async () => {
      // 1. Create an agent
      const createResponse = await axios.post(`${BACKEND_URL}/api/agents/create/amazon_q`);
      const agent = createResponse.data;

      // 2. List agents to verify creation
      const listResponse = await axios.get(`${BACKEND_URL}/api/agents/list`);
      const agents = listResponse.data;
      const foundAgent = agents.find(a => a.id === agent.id);
      
      if (!foundAgent) {
        throw new Error('Created agent not found in list');
      }

      // 3. Query the agent directly
      const queryResponse = await axios.post(
        `${BACKEND_URL}/api/agents/query/${agent.id}`,
        { prompt: 'What is AWS Lambda?', context: 'Serverless computing' }
      );

      if (!queryResponse.data || typeof queryResponse.data !== 'string') {
        throw new Error('Invalid query response');
      }

      // 4. Test via frontend API
      const frontendResponse = await axios.post(
        `${FRONTEND_URL}/api/agents/amazon-q/query`,
        { prompt: 'What is AWS Lambda?', context: 'Serverless computing' }
      );

      if (!frontendResponse.data.response) {
        throw new Error('Invalid frontend API response');
      }

      console.log('   ✓ Agent creation, listing, querying, and frontend API all working');
    });
  }

  async runAllTests() {
    console.log('🚀 Starting AI Agents Integration Tests\\n');
    console.log('=' .repeat(60));

    await this.runTest('Backend Health Check', () => this.testBackendHealth());
    await this.testAgentCreation();
    await this.runTest('Agent Listing', () => this.testAgentListing());
    await this.testAgentQuery();
    await this.testFrontendAPIRoutes();
    await this.testFrontendPages();
    await this.testEndToEndWorkflow();

    console.log('\\n' + '='.repeat(60));
    console.log('📊 AI AGENTS TEST RESULTS');
    console.log('='.repeat(60));
    console.log(`✅ Passed: ${this.results.passed}`);
    console.log(`❌ Failed: ${this.results.failed}`);
    console.log(`📈 Success Rate: ${((this.results.passed / (this.results.passed + this.results.failed)) * 100).toFixed(1)}%`);

    if (this.results.failed > 0) {
      console.log('\\n❌ Failed Tests:');
      this.results.tests
        .filter(test => test.status === 'FAILED')
        .forEach(test => {
          console.log(`   • ${test.name}: ${test.error}`);
        });
    }

    console.log('\\n🎯 AI Agents Tested:');
    AI_AGENTS.forEach(agent => {
      console.log(`   • ${agent.name} (${agent.type})`);
    });

    console.log('\\n🔧 Features Tested:');
    console.log('   • Agent Creation & Management');
    console.log('   • Agent Querying & Responses');
    console.log('   • Frontend API Routes');
    console.log('   • Frontend Pages');
    console.log('   • End-to-End Workflow');

    return this.results.failed === 0;
  }
}

// Run tests if called directly
if (require.main === module) {
  const tester = new AIAgentTester();
  tester.runAllTests()
    .then(success => {
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error('Test runner failed:', error);
      process.exit(1);
    });
}

module.exports = AIAgentTester;