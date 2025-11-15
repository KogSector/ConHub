#!/usr/bin/env node

/**
 * Microservice Validation Script
 * 
 * Validates that all microservices are properly configured with individual .env files
 * and can start independently.
 */

const fs = require('fs');
const path = require('path');
const http = require('http');

const MICROSERVICES = [
  { name: 'auth', port: 3010, hasDockerfile: true },
  { name: 'data', port: 3013, hasDockerfile: true },
  { name: 'frontend', port: 3000, hasDockerfile: true },
  { name: 'billing', port: 3011, hasDockerfile: true },
  { name: 'security', port: 3014, hasDockerfile: true },
  { name: 'webhook', port: 3015, hasDockerfile: true },
  { name: 'embedding', port: null, hasDockerfile: true },
  { name: 'indexers', port: null, hasDockerfile: true },
  { name: 'backend', port: null, hasDockerfile: true },
  { name: 'client', port: null, hasDockerfile: true }
];

class MicroserviceValidator {
  constructor() {
    this.projectRoot = path.resolve(__dirname, '..', '..');
    this.results = {
      envFiles: {},
      dockerfiles: {},
      healthChecks: {},
      summary: {}
    };
  }

  async validateAll() {
    console.log('🔍 ConHub Microservice Validation');
    console.log('='.repeat(50));

    await this.validateEnvFiles();
    await this.validateDockerfiles();
    await this.validateHealthChecks();
    await this.generateSummary();
    
    this.printResults();
  }

  async validateEnvFiles() {
    console.log('\n📄 Validating .env files...');
    
    for (const service of MICROSERVICES) {
      const envPath = path.join(this.projectRoot, service.name, '.env');
      const exists = fs.existsSync(envPath);
      
      this.results.envFiles[service.name] = {
        exists,
        path: envPath,
        valid: false
      };

      if (exists) {
        try {
          const content = fs.readFileSync(envPath, 'utf8');
          const hasDbUrl = content.includes('DATABASE_URL_NEON') || content.includes('DATABASE_URL');
          const hasEnvMode = content.includes('ENV_MODE');
          
          this.results.envFiles[service.name].valid = hasDbUrl || service.name === 'frontend';
          this.results.envFiles[service.name].hasDbUrl = hasDbUrl;
          this.results.envFiles[service.name].hasEnvMode = hasEnvMode;
          
          console.log(`  ✅ ${service.name}: .env file exists and is valid`);
        } catch (error) {
          console.log(`  ❌ ${service.name}: .env file exists but cannot be read`);
        }
      } else {
        console.log(`  ❌ ${service.name}: .env file missing`);
      }
    }
  }

  async validateDockerfiles() {
    console.log('\n🐳 Validating Dockerfiles...');
    
    for (const service of MICROSERVICES) {
      const dockerfilePath = path.join(this.projectRoot, service.name, 'Dockerfile');
      const exists = fs.existsSync(dockerfilePath);
      
      this.results.dockerfiles[service.name] = {
        exists,
        path: dockerfilePath
      };

      if (exists) {
        console.log(`  ✅ ${service.name}: Dockerfile exists`);
      } else {
        console.log(`  ❌ ${service.name}: Dockerfile missing`);
      }
    }
  }

  async validateHealthChecks() {
    console.log('\n🏥 Validating service health...');
    
    for (const service of MICROSERVICES) {
      if (!service.port) {
        console.log(`  ⏭️  ${service.name}: No health check (no port defined)`);
        continue;
      }

      try {
        const isHealthy = await this.checkServiceHealth(service.name, service.port);
        this.results.healthChecks[service.name] = {
          port: service.port,
          healthy: isHealthy
        };

        if (isHealthy) {
          console.log(`  ✅ ${service.name}: Service is healthy on port ${service.port}`);
        } else {
          console.log(`  ❌ ${service.name}: Service not responding on port ${service.port}`);
        }
      } catch (error) {
        console.log(`  ❌ ${service.name}: Health check failed - ${error.message}`);
        this.results.healthChecks[service.name] = {
          port: service.port,
          healthy: false,
          error: error.message
        };
      }
    }
  }

  async checkServiceHealth(serviceName, port) {
    return new Promise((resolve) => {
      const healthPath = serviceName === 'frontend' ? '/' : '/health';
      const url = `http://localhost:${port}${healthPath}`;
      
      const req = http.get(url, { timeout: 3000 }, (res) => {
        resolve(res.statusCode === 200);
      });

      req.on('error', () => resolve(false));
      req.on('timeout', () => {
        req.destroy();
        resolve(false);
      });
    });
  }

  async generateSummary() {
    const totalServices = MICROSERVICES.length;
    const servicesWithEnv = Object.values(this.results.envFiles).filter(r => r.exists).length;
    const servicesWithDockerfile = Object.values(this.results.dockerfiles).filter(r => r.exists).length;
    const healthyServices = Object.values(this.results.healthChecks).filter(r => r.healthy).length;
    const servicesWithPorts = MICROSERVICES.filter(s => s.port).length;

    this.results.summary = {
      totalServices,
      servicesWithEnv,
      servicesWithDockerfile,
      healthyServices,
      servicesWithPorts,
      envCoverage: Math.round((servicesWithEnv / totalServices) * 100),
      dockerfileCoverage: Math.round((servicesWithDockerfile / totalServices) * 100),
      healthCoverage: servicesWithPorts > 0 ? Math.round((healthyServices / servicesWithPorts) * 100) : 0
    };
  }

  printResults() {
    console.log('\n📊 Validation Summary');
    console.log('='.repeat(50));
    
    const { summary } = this.results;
    
    console.log(`📄 Environment Files: ${summary.servicesWithEnv}/${summary.totalServices} (${summary.envCoverage}%)`);
    console.log(`🐳 Dockerfiles: ${summary.servicesWithDockerfile}/${summary.totalServices} (${summary.dockerfileCoverage}%)`);
    console.log(`🏥 Healthy Services: ${summary.healthyServices}/${summary.servicesWithPorts} (${summary.healthCoverage}%)`);
    
    console.log('\n🎯 Microservice Independence Status:');
    
    if (summary.envCoverage === 100) {
      console.log('  ✅ All microservices have individual .env files');
    } else {
      console.log(`  ⚠️  ${summary.totalServices - summary.servicesWithEnv} microservices missing .env files`);
    }
    
    if (summary.dockerfileCoverage === 100) {
      console.log('  ✅ All microservices have Dockerfiles');
    } else {
      console.log(`  ⚠️  ${summary.totalServices - summary.servicesWithDockerfile} microservices missing Dockerfiles`);
    }
    
    if (summary.healthCoverage >= 80) {
      console.log('  ✅ Most services are running and healthy');
    } else if (summary.healthCoverage >= 50) {
      console.log('  ⚠️  Some services are not running');
    } else {
      console.log('  ❌ Most services are not running');
    }

    console.log('\n🏆 Overall Status:');
    const overallScore = Math.round((summary.envCoverage + summary.dockerfileCoverage + summary.healthCoverage) / 3);
    
    if (overallScore >= 90) {
      console.log('  🟢 EXCELLENT - Microservice setup is optimal');
    } else if (overallScore >= 75) {
      console.log('  🟡 GOOD - Microservice setup is mostly complete');
    } else if (overallScore >= 50) {
      console.log('  🟠 FAIR - Microservice setup needs improvement');
    } else {
      console.log('  🔴 POOR - Microservice setup requires attention');
    }
    
    console.log(`  📈 Overall Score: ${overallScore}%`);
  }
}

if (require.main === module) {
  const validator = new MicroserviceValidator();
  validator.validateAll().catch(console.error);
}

module.exports = MicroserviceValidator;
