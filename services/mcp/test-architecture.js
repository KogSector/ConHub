#!/usr/bin/env node

/**
 * Test script to verify the new MCP connector architecture
 */

import path from 'path';
import fs from 'fs/promises';
import { fileURLToPath } from 'url';

// Import the connector loader and base connector
import ConnectorLoader from './service/src/connectors/loader.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class ArchitectureTest {
  constructor() {
    this.loader = null;
    this.testResults = [];
  }

  async runTests() {
    console.log('🧪 Starting MCP Architecture Tests...\n');

    try {
      // Test 1: Connector Loader Initialization
      await this.testConnectorLoader();

      // Test 2: Base Connector Functionality
      await this.testBaseConnector();

      // Test 3: Connector Discovery
      await this.testConnectorDiscovery();

      // Test 4: Connector Loading
      await this.testConnectorLoading();

      // Test 5: Health Checks
      await this.testHealthChecks();

      // Test 6: Cleanup
      await this.testCleanup();

      // Generate test report
      await this.generateTestReport();

      console.log('\n✅ All tests completed!');
      this.printTestSummary();

    } catch (error) {
      console.error('\n❌ Test suite failed:', error.message);
      process.exit(1);
    }
  }

  async testConnectorLoader() {
    console.log('📦 Testing Connector Loader...');
    
    try {
      this.loader = new ConnectorLoader();
      
      // Test initialization
      await this.loader.initialize();
      
      this.addTestResult('ConnectorLoader Initialization', true, 'Loader initialized successfully');
      console.log('  ✅ Loader initialization passed');
      
    } catch (error) {
      this.addTestResult('ConnectorLoader Initialization', false, error.message);
      console.log('  ❌ Loader initialization failed:', error.message);
      throw error;
    }
  }

  async testBaseConnector() {
    console.log('\n🔧 Testing Base Connector...');
    
    try {
      // Check if BaseConnector file exists
      const baseConnectorPath = path.join(__dirname, 'connectors', 'BaseConnector.js');
      
      try {
        await fs.access(baseConnectorPath);
        this.addTestResult('BaseConnector File Exists', true, 'BaseConnector.js file found');
        console.log('  ✅ BaseConnector.js file exists');
        
        // Read the file to check for key methods
        const content = await fs.readFile(baseConnectorPath, 'utf8');
        
        if (content.includes('log(')) {
          console.log('  ✅ Log method found in BaseConnector');
        }
        if (content.includes('validateConfig')) {
          console.log('  ✅ Config validation method found in BaseConnector');
        }
        if (content.includes('validateInitialized')) {
          console.log('  ✅ Initialization check method found in BaseConnector');
        }
        
      } catch (error) {
        throw new Error('BaseConnector.js file not found');
      }
      
    } catch (error) {
      this.addTestResult('BaseConnector Functionality', false, error.message);
      console.log('  ❌ Base connector test failed:', error.message);
      // Don't throw here as this is not critical for the architecture test
    }
  }

  async testConnectorDiscovery() {
    console.log('\n🔍 Testing Connector Discovery...');
    
    try {
      const connectorsDir = path.join(__dirname, 'connectors');
      const connectors = await this.loader.discoverConnectors(connectorsDir);
      
      if (connectors.length > 0) {
        this.addTestResult('Connector Discovery', true, `Found ${connectors.length} connectors`);
        console.log(`  ✅ Discovered ${connectors.length} connectors:`);
        
        for (const connector of connectors) {
          console.log(`    - ${connector.name} (${connector.path})`);
        }
      } else {
        throw new Error('No connectors discovered');
      }
      
    } catch (error) {
      this.addTestResult('Connector Discovery', false, error.message);
      console.log('  ❌ Connector discovery failed:', error.message);
      throw error;
    }
  }

  async testConnectorLoading() {
    console.log('\n⚡ Testing Connector Loading...');
    
    try {
      // Test loading filesystem connector (should work without external dependencies)
      const filesystemPath = path.join(__dirname, 'connectors', 'filesystem');
      
      if (await this.directoryExists(filesystemPath)) {
        await this.loader.loadConnector('filesystem');
        const connector = this.loader.getConnector('filesystem');
        
        if (connector) {
          this.addTestResult('Filesystem Connector Loading', true, 'Filesystem connector loaded successfully');
          console.log('  ✅ Filesystem connector loaded successfully');
          
          // Test connector methods
          if (typeof connector.register === 'function') {
            console.log('  ✅ Register method exists');
          }
          if (typeof connector.initialize === 'function') {
            console.log('  ✅ Initialize method exists');
          }
          if (typeof connector.healthCheck === 'function') {
            console.log('  ✅ Health check method exists');
          }
          
        } else {
          throw new Error('Connector loading returned null');
        }
      } else {
        this.addTestResult('Filesystem Connector Loading', false, 'Filesystem connector directory not found');
        console.log('  ⚠️  Filesystem connector directory not found, skipping load test');
      }
      
    } catch (error) {
      this.addTestResult('Connector Loading', false, error.message);
      console.log('  ❌ Connector loading failed:', error.message);
      // Don't throw here as this might fail due to missing dependencies
    }
  }

  async testHealthChecks() {
    console.log('\n🏥 Testing Health Checks...');
    
    try {
      const health = await this.loader.getHealth();
      
      if (health && typeof health === 'object') {
        this.addTestResult('Loader Health Check', true, `Health status: ${health.status}`);
        console.log(`  ✅ Loader health check passed: ${health.status}`);
        
        if (health.connectors) {
          console.log(`  📊 Connector health summary:`);
          for (const [name, connectorHealth] of Object.entries(health.connectors)) {
            console.log(`    - ${name}: ${connectorHealth.status}`);
          }
        }
      } else {
        throw new Error('Invalid health check response');
      }
      
    } catch (error) {
      this.addTestResult('Health Checks', false, error.message);
      console.log('  ❌ Health check failed:', error.message);
    }
  }

  async testCleanup() {
    console.log('\n🧹 Testing Cleanup...');
    
    try {
      await this.loader.cleanup();
      
      this.addTestResult('Loader Cleanup', true, 'Cleanup completed successfully');
      console.log('  ✅ Cleanup completed successfully');
      
    } catch (error) {
      this.addTestResult('Cleanup', false, error.message);
      console.log('  ❌ Cleanup failed:', error.message);
    }
  }

  async generateTestReport() {
    const report = {
      timestamp: new Date().toISOString(),
      summary: {
        total: this.testResults.length,
        passed: this.testResults.filter(r => r.passed).length,
        failed: this.testResults.filter(r => !r.passed).length
      },
      tests: this.testResults
    };
    
    await fs.writeFile(
      path.join(__dirname, 'test-report.json'),
      JSON.stringify(report, null, 2)
    );
  }

  printTestSummary() {
    const total = this.testResults.length;
    const passed = this.testResults.filter(r => r.passed).length;
    const failed = this.testResults.filter(r => !r.passed).length;
    
    console.log('\n📊 Test Summary:');
    console.log(`  Total Tests: ${total}`);
    console.log(`  Passed: ${passed} ✅`);
    console.log(`  Failed: ${failed} ❌`);
    console.log(`  Success Rate: ${((passed / total) * 100).toFixed(1)}%`);
    
    if (failed > 0) {
      console.log('\n❌ Failed Tests:');
      this.testResults
        .filter(r => !r.passed)
        .forEach(test => {
          console.log(`  - ${test.name}: ${test.message}`);
        });
    }
    
    console.log(`\n📄 Detailed report saved to: ${path.join(__dirname, 'test-report.json')}`);
  }

  addTestResult(name, passed, message) {
    this.testResults.push({
      name,
      passed,
      message,
      timestamp: new Date().toISOString()
    });
  }

  async directoryExists(dirPath) {
    try {
      const stats = await fs.stat(dirPath);
      return stats.isDirectory();
    } catch {
      return false;
    }
  }
}

// Run tests if this script is executed directly
const currentFile = fileURLToPath(import.meta.url);
const isMainModule = process.argv[1] === currentFile;

if (isMainModule) {
  const tester = new ArchitectureTest();
  tester.runTests().catch(console.error);
}

export default ArchitectureTest;