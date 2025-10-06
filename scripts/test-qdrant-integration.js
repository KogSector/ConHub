#!/usr/bin/env node

/**
 * Qdrant Vector Database Integration Test
 */

const axios = require('axios');

const AI_SERVICE_URL = 'http://localhost:8001';

class QdrantTester {
  constructor() {
    this.results = { passed: 0, failed: 0, tests: [] };
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

  async testQdrantConnection() {
    const response = await axios.get(`${AI_SERVICE_URL}/qdrant/collections/info`);
    if (response.status !== 200) {
      throw new Error(`Expected 200, got ${response.status}`);
    }
    console.log(`   Collections: ${Object.keys(response.data.collections).join(', ')}`);
  }

  async testAddCodeVectors() {
    const FormData = require('form-data');
    const form = new FormData();
    form.append('content', 'function hello() { console.log("Hello World"); }');
    form.append('file_path', '/src/hello.js');
    form.append('language', 'javascript');
    form.append('repository', 'test-repo');

    const response = await axios.post(`${AI_SERVICE_URL}/qdrant/vectors/add/code`, form, {
      headers: form.getHeaders(),
      timeout: 10000
    });

    if (response.status !== 200 || !response.data.success) {
      throw new Error('Failed to add code vectors');
    }
  }

  async testSearchVectors() {
    const FormData = require('form-data');
    const form = new FormData();
    form.append('query', 'hello function javascript');
    form.append('collection_type', 'code');
    form.append('limit', '5');

    const response = await axios.post(`${AI_SERVICE_URL}/qdrant/search`, form, {
      headers: form.getHeaders(),
      timeout: 10000
    });

    if (response.status !== 200 || !response.data.success) {
      throw new Error('Failed to search vectors');
    }

    console.log(`   Found ${response.data.results.length} results`);
  }

  async runAllTests() {
    console.log('🚀 Starting Qdrant Integration Tests\n');
    console.log('=' .repeat(50));

    await this.runTest('Qdrant Connection', () => this.testQdrantConnection());
    await this.runTest('Add Code Vectors', () => this.testAddCodeVectors());
    await this.runTest('Search Vectors', () => this.testSearchVectors());

    console.log('\n' + '='.repeat(50));
    console.log('📊 QDRANT TEST RESULTS');
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

    return this.results.failed === 0;
  }
}

if (require.main === module) {
  const tester = new QdrantTester();
  tester.runAllTests()
    .then(success => {
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error('Test runner failed:', error);
      process.exit(1);
    });
}

module.exports = QdrantTester;