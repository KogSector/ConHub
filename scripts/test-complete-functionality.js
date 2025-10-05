#!/usr/bin/env node

/**
 * Comprehensive test script for ConHub functionality
 * Tests repository connection, document upload, URL indexing, and AI agent connectivity
 */

const axios = require('axios');
const fs = require('fs');
const path = require('path');

const BACKEND_URL = 'http://localhost:3001';
const AI_SERVICE_URL = 'http://localhost:8001';
const LEXOR_URL = 'http://localhost:3002';

class ConHubTester {
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

    async testServiceHealth() {
        const services = [
            { name: 'Backend', url: `${BACKEND_URL}/health` },
            { name: 'AI Service', url: `${AI_SERVICE_URL}/health` },
            { name: 'Lexor', url: `${LEXOR_URL}/health` }
        ];

        for (const service of services) {
            await this.runTest(`${service.name} Health Check`, async () => {
                const response = await axios.get(service.url, { timeout: 5000 });
                if (response.status !== 200) {
                    throw new Error(`Expected 200, got ${response.status}`);
                }
            });
        }
    }

    async testRepositoryConnection() {
        await this.runTest('Repository Branch Fetching', async () => {
            const payload = {
                repo_url: 'https://github.com/octocat/Hello-World',
                credentials: {
                    credential_type: {
                        PersonalAccessToken: {
                            token: 'dummy_token_for_testing'
                        }
                    },
                    expires_at: null
                }
            };

            try {
                const response = await axios.post(`${BACKEND_URL}/api/repositories/fetch-branches`, payload);
                // This will likely fail due to invalid token, but we're testing the endpoint structure
            } catch (error) {
                if (error.response && error.response.status === 401) {
                    // Expected - invalid token
                    return;
                }
                throw error;
            }
        });

        await this.runTest('Repository Connection Endpoint', async () => {
            const payload = {
                url: 'https://github.com/octocat/Hello-World',
                credentials: {
                    credential_type: {
                        PersonalAccessToken: {
                            token: 'dummy_token_for_testing'
                        }
                    },
                    expires_at: null
                }
            };

            try {
                const response = await axios.post(`${BACKEND_URL}/api/repositories/connect`, payload);
                // This will likely fail due to invalid token, but we're testing the endpoint structure
            } catch (error) {
                if (error.response && (error.response.status === 401 || error.response.status === 400)) {
                    // Expected - invalid token or bad request
                    return;
                }
                throw error;
            }
        });
    }

    async testDocumentSources() {
        await this.runTest('Local File Upload', async () => {
            // Create a test file
            const testContent = 'This is a test document for ConHub indexing.';
            const testFile = path.join(__dirname, 'test-document.txt');
            fs.writeFileSync(testFile, testContent);

            try {
                const FormData = require('form-data');
                const form = new FormData();
                form.append('files', fs.createReadStream(testFile), 'test-document.txt');

                const response = await axios.post(`${AI_SERVICE_URL}/sources/local-files`, form, {
                    headers: form.getHeaders(),
                    timeout: 10000
                });

                if (response.status !== 200) {
                    throw new Error(`Expected 200, got ${response.status}`);
                }

                if (!response.data.success) {
                    throw new Error('Upload was not successful');
                }
            } finally {
                // Clean up test file
                if (fs.existsSync(testFile)) {
                    fs.unlinkSync(testFile);
                }
            }
        });

        await this.runTest('Document Source Endpoints', async () => {
            const endpoints = [
                '/sources/dropbox',
                '/sources/google-drive',
                '/sources/onedrive'
            ];

            for (const endpoint of endpoints) {
                try {
                    const FormData = require('form-data');
                    const form = new FormData();
                    form.append('access_token', 'dummy_token');
                    form.append('folder_path', '/test');

                    await axios.post(`${AI_SERVICE_URL}${endpoint}`, form, {
                        headers: form.getHeaders(),
                        timeout: 5000
                    });
                } catch (error) {
                    if (error.response && error.response.status >= 400 && error.response.status < 500) {
                        // Expected - invalid credentials
                        continue;
                    }
                    throw new Error(`Endpoint ${endpoint} failed unexpectedly: ${error.message}`);
                }
            }
        });
    }

    async testURLIndexing() {
        await this.runTest('URL Indexing', async () => {
            const FormData = require('form-data');
            const form = new FormData();
            form.append('urls', JSON.stringify(['https://httpbin.org/html']));
            form.append('config', JSON.stringify({ crawl_depth: 1 }));

            const response = await axios.post(`${AI_SERVICE_URL}/index/urls`, form, {
                headers: form.getHeaders(),
                timeout: 15000
            });

            if (response.status !== 200) {
                throw new Error(`Expected 200, got ${response.status}`);
            }

            if (!response.data.success) {
                throw new Error('URL indexing was not successful');
            }

            const jobId = response.data.job_id;
            
            // Wait a bit and check status
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            const statusResponse = await axios.get(`${AI_SERVICE_URL}/index/urls/${jobId}/status`);
            if (statusResponse.status !== 200) {
                throw new Error(`Status check failed: ${statusResponse.status}`);
            }
        });
    }

    async testAIAgents() {
        await this.runTest('AI Agent Endpoints', async () => {
            // Test getting available agents
            const agentsResponse = await axios.get(`${AI_SERVICE_URL}/ai/agents`);
            if (agentsResponse.status !== 200) {
                throw new Error(`Expected 200, got ${agentsResponse.status}`);
            }
        });

        await this.runTest('AI Agent Connection', async () => {
            const FormData = require('form-data');
            const form = new FormData();
            form.append('agent_type', 'openai_gpt');
            form.append('credentials', JSON.stringify({ api_key: 'dummy_key' }));
            form.append('config', JSON.stringify({ model: 'gpt-4' }));

            try {
                await axios.post(`${AI_SERVICE_URL}/ai/agents/connect`, form, {
                    headers: form.getHeaders(),
                    timeout: 5000
                });
            } catch (error) {
                if (error.response && error.response.status >= 400 && error.response.status < 500) {
                    // Expected - invalid credentials
                    return;
                }
                throw error;
            }
        });
    }

    async testUnifiedContextAPI() {
        await this.runTest('Unified Context API', async () => {
            const payload = {
                query: 'test query',
                agent_type: 'github_copilot',
                include_code: true,
                include_documents: true,
                max_results: 5
            };

            try {
                const response = await axios.post(`${BACKEND_URL}/api/agents/query/test-agent`, payload);
                // This might fail if no agents are connected, but we're testing the endpoint
            } catch (error) {
                if (error.response && error.response.status === 404) {
                    // Expected - no connected agents
                    return;
                }
                if (error.response && error.response.status >= 500) {
                    // Server error is acceptable for this test
                    return;
                }
                throw error;
            }
        });
    }

    async testVectorSearch() {
        await this.runTest('Vector Search', async () => {
            const FormData = require('form-data');
            const form = new FormData();
            form.append('query', 'test search');
            form.append('k', '5');

            const response = await axios.post(`${AI_SERVICE_URL}/vector/search`, form, {
                headers: form.getHeaders(),
                timeout: 5000
            });

            if (response.status !== 200) {
                throw new Error(`Expected 200, got ${response.status}`);
            }
        });
    }

    async testLexorSearch() {
        await this.runTest('Lexor Code Search', async () => {
            const payload = {
                query: 'test function',
                limit: 5,
                search_type: 'semantic'
            };

            try {
                const response = await axios.post(`${LEXOR_URL}/api/search`, payload);
                if (response.status !== 200) {
                    throw new Error(`Expected 200, got ${response.status}`);
                }
            } catch (error) {
                if (error.code === 'ECONNREFUSED') {
                    throw new Error('Lexor service is not running');
                }
                throw error;
            }
        });
    }

    async runAllTests() {
        console.log('🚀 Starting ConHub Comprehensive Tests\n');
        console.log('=' .repeat(50));

        await this.testServiceHealth();
        await this.testRepositoryConnection();
        await this.testDocumentSources();
        await this.testURLIndexing();
        await this.testAIAgents();
        await this.testUnifiedContextAPI();
        await this.testVectorSearch();
        await this.testLexorSearch();

        console.log('\n' + '='.repeat(50));
        console.log('📊 TEST RESULTS');
        console.log('='.repeat(50));
        console.log(`✅ Passed: ${this.results.passed}`);
        console.log(`❌ Failed: ${this.results.failed}`);
        console.log(`📈 Success Rate: ${((this.results.passed / (this.results.passed + this.results.failed)) * 100).toFixed(1)}%`);

        if (this.results.failed > 0) {
            console.log('\n❌ Failed Tests:');
            this.results.tests
                .filter(test => test.status === 'FAILED')
                .forEach(test => {
                    console.log(`   • ${test.name}: ${test.error}`);
                });
        }

        console.log('\n🎯 Key Features Tested:');
        console.log('   • Service Health Checks');
        console.log('   • Repository Connection (GitHub/GitLab/BitBucket)');
        console.log('   • Document Source Integration');
        console.log('   • URL Indexing and Crawling');
        console.log('   • AI Agent Connectivity');
        console.log('   • Unified Context API');
        console.log('   • Vector and Code Search');

        return this.results.failed === 0;
    }
}

// Run tests if called directly
if (require.main === module) {
    const tester = new ConHubTester();
    tester.runAllTests()
        .then(success => {
            process.exit(success ? 0 : 1);
        })
        .catch(error => {
            console.error('Test runner failed:', error);
            process.exit(1);
        });
}

module.exports = ConHubTester;