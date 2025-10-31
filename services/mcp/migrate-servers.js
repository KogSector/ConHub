#!/usr/bin/env node

/**
 * Migration script to convert existing MCP servers to the new connector architecture
 */

const fs = require('fs').promises;
const path = require('path');

class ServerMigrator {
  constructor() {
    this.serversDir = path.join(__dirname, 'servers');
    this.connectorsDir = path.join(__dirname, 'connectors');
    this.migrationLog = [];
  }

  async migrate() {
    console.log('🚀 Starting MCP server migration...');
    
    try {
      // Ensure connectors directory exists
      await this.ensureDirectory(this.connectorsDir);
      
      // Get list of existing servers
      const servers = await this.getExistingServers();
      console.log(`📁 Found ${servers.length} servers to migrate`);
      
      // Migrate each server
      for (const server of servers) {
        await this.migrateServer(server);
      }
      
      // Generate migration report
      await this.generateMigrationReport();
      
      console.log('✅ Migration completed successfully!');
      console.log(`📊 Migration report saved to: ${path.join(__dirname, 'migration-report.json')}`);
      
    } catch (error) {
      console.error('❌ Migration failed:', error.message);
      process.exit(1);
    }
  }

  async getExistingServers() {
    const servers = [];
    
    try {
      const sourcesDir = path.join(this.serversDir, 'sources');
      const agentsDir = path.join(this.serversDir, 'agents');
      
      // Check sources directory
      if (await this.directoryExists(sourcesDir)) {
        const sourceServers = await fs.readdir(sourcesDir);
        for (const server of sourceServers) {
          const serverPath = path.join(sourcesDir, server);
          if ((await fs.stat(serverPath)).isDirectory()) {
            servers.push({
              name: server,
              type: 'source',
              path: serverPath,
              category: 'data-source'
            });
          }
        }
      }
      
      // Check agents directory
      if (await this.directoryExists(agentsDir)) {
        const agentServers = await fs.readdir(agentsDir);
        for (const server of agentServers) {
          const serverPath = path.join(agentsDir, server);
          if ((await fs.stat(serverPath)).isDirectory()) {
            servers.push({
              name: server,
              type: 'agent',
              path: serverPath,
              category: 'ai-agent'
            });
          }
        }
      }
      
    } catch (error) {
      console.warn('⚠️  Warning: Could not read servers directory:', error.message);
    }
    
    return servers;
  }

  async migrateServer(server) {
    console.log(`🔄 Migrating ${server.type} server: ${server.name}`);
    
    try {
      const connectorDir = path.join(this.connectorsDir, server.name);
      
      // Skip if connector already exists
      if (await this.directoryExists(connectorDir)) {
        console.log(`⏭️  Skipping ${server.name} - connector already exists`);
        this.migrationLog.push({
          server: server.name,
          status: 'skipped',
          reason: 'Connector already exists'
        });
        return;
      }
      
      // Create connector directory
      await this.ensureDirectory(connectorDir);
      
      // Read existing server configuration
      const serverConfig = await this.readServerConfig(server);
      
      // Generate connector configuration
      const connectorConfig = await this.generateConnectorConfig(server, serverConfig);
      
      // Write connector config
      await fs.writeFile(
        path.join(connectorDir, 'config.json'),
        JSON.stringify(connectorConfig, null, 2)
      );
      
      // Generate connector implementation
      const connectorImpl = await this.generateConnectorImplementation(server, serverConfig);
      
      // Write connector implementation
      await fs.writeFile(
        path.join(connectorDir, 'index.js'),
        connectorImpl
      );
      
      // Copy additional files if needed
      await this.copyAdditionalFiles(server, connectorDir);
      
      console.log(`✅ Successfully migrated ${server.name}`);
      this.migrationLog.push({
        server: server.name,
        status: 'success',
        connectorPath: connectorDir
      });
      
    } catch (error) {
      console.error(`❌ Failed to migrate ${server.name}:`, error.message);
      this.migrationLog.push({
        server: server.name,
        status: 'failed',
        error: error.message
      });
    }
  }

  async readServerConfig(server) {
    const packageJsonPath = path.join(server.path, 'package.json');
    const serverJsPath = path.join(server.path, 'server.js');
    
    let packageJson = {};
    let serverJs = '';
    
    try {
      if (await this.fileExists(packageJsonPath)) {
        const packageData = await fs.readFile(packageJsonPath, 'utf8');
        packageJson = JSON.parse(packageData);
      }
    } catch (error) {
      console.warn(`⚠️  Could not read package.json for ${server.name}`);
    }
    
    try {
      if (await this.fileExists(serverJsPath)) {
        serverJs = await fs.readFile(serverJsPath, 'utf8');
      }
    } catch (error) {
      console.warn(`⚠️  Could not read server.js for ${server.name}`);
    }
    
    return { packageJson, serverJs };
  }

  async generateConnectorConfig(server, serverConfig) {
    const baseConfig = {
      id: server.name,
      name: this.capitalizeWords(server.name.replace(/-/g, ' ')),
      version: serverConfig.packageJson.version || '1.0.0',
      description: serverConfig.packageJson.description || `${server.name} connector`,
      capabilities: this.inferCapabilities(server, serverConfig),
      metadata: {
        category: server.category,
        provider: server.name,
        supportedFormats: this.inferSupportedFormats(server),
        rateLimits: {
          requestsPerMinute: 60,
          requestsPerHour: 1000
        }
      },
      authentication: this.inferAuthentication(server, serverConfig),
      settings: this.inferSettings(server, serverConfig)
    };
    
    return baseConfig;
  }

  inferCapabilities(server, serverConfig) {
    const capabilities = [];
    
    // Common capabilities based on server type
    if (server.type === 'source') {
      capabilities.push('search', 'metadata');
      
      // Infer specific capabilities based on server name
      if (server.name.includes('drive') || server.name.includes('dropbox')) {
        capabilities.push('files', 'folders', 'download');
      } else if (server.name.includes('filesystem')) {
        capabilities.push('files', 'directories', 'read', 'write');
      } else {
        capabilities.push('documents', 'files');
      }
    } else if (server.type === 'agent') {
      capabilities.push('chat', 'completion');
      
      if (server.name.includes('code') || server.name.includes('copilot')) {
        capabilities.push('code-generation', 'code-analysis');
      }
    }
    
    return capabilities;
  }

  inferSupportedFormats(server) {
    if (server.name.includes('drive') || server.name.includes('dropbox')) {
      return ['documents', 'spreadsheets', 'presentations', 'images', 'text'];
    } else if (server.name.includes('filesystem')) {
      return ['text', 'code', 'documents', 'images', 'binary'];
    } else {
      return ['text', 'documents'];
    }
  }

  inferAuthentication(server, serverConfig) {
    // Check for OAuth patterns in server code
    if (serverConfig.serverJs.includes('oauth') || serverConfig.serverJs.includes('OAuth')) {
      return {
        type: 'oauth2',
        scopes: this.inferOAuthScopes(server)
      };
    } else if (serverConfig.serverJs.includes('apiKey') || serverConfig.serverJs.includes('API_KEY')) {
      return {
        type: 'api-key'
      };
    } else {
      return {
        type: 'none'
      };
    }
  }

  inferOAuthScopes(server) {
    if (server.name.includes('google-drive')) {
      return ['https://www.googleapis.com/auth/drive.readonly'];
    } else if (server.name.includes('dropbox')) {
      return ['files.metadata.read', 'files.content.read'];
    } else {
      return [];
    }
  }

  inferSettings(server, serverConfig) {
    const settings = {
      maxFileSize: '100MB'
    };
    
    if (server.type === 'source') {
      if (server.name.includes('filesystem')) {
        settings.allowedPaths = ['./'];
        settings.supportedOperations = ['read', 'write', 'list', 'search'];
      } else {
        settings.supportedOperations = ['read', 'list', 'search', 'download'];
      }
    }
    
    return settings;
  }

  async generateConnectorImplementation(server, serverConfig) {
    const template = `const BaseConnector = require('../BaseConnector');

class ${this.toPascalCase(server.name)}Connector extends BaseConnector {
  constructor() {
    super();
    this.config = null;
    // TODO: Add connector-specific properties
  }

  async register(core) {
    try {
      // Load connector configuration
      const configPath = require('path').join(__dirname, 'config.json');
      const configData = require('fs').readFileSync(configPath, 'utf8');
      this.config = JSON.parse(configData);

      // Register with core MCP service
      await core.registerConnector(this.config.id, {
        name: this.config.name,
        version: this.config.version,
        capabilities: this.config.capabilities,
        metadata: this.config.metadata,
        connector: this
      });

      this.log('info', '${this.capitalizeWords(server.name)} connector registered successfully');
      return true;
    } catch (error) {
      this.log('error', 'Failed to register ${server.name} connector', { error: error.message });
      throw error;
    }
  }

  async initialize(config = {}) {
    try {
      // TODO: Implement initialization logic
      // This should include setting up API clients, validating credentials, etc.
      
      this.initialized = true;
      this.log('info', '${this.capitalizeWords(server.name)} connector initialized');
      return true;
    } catch (error) {
      this.log('error', 'Failed to initialize ${server.name} connector', { error: error.message });
      throw error;
    }
  }

  async healthCheck() {
    try {
      if (!this.initialized) {
        return { status: 'unhealthy', message: 'Connector not initialized' };
      }

      // TODO: Implement health check logic
      // This should verify API connectivity, authentication status, etc.
      
      return {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        details: {
          // Add relevant health check details
        }
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        message: error.message,
        timestamp: new Date().toISOString()
      };
    }
  }

  async authenticate(credentials) {
    try {
      // TODO: Implement authentication logic
      // This should handle OAuth, API keys, or other auth methods
      
      return { success: true };
    } catch (error) {
      this.log('error', 'Authentication failed', { error: error.message });
      throw error;
    }
  }

  async fetchData(query) {
    try {
      this.validateInitialized();
      
      // TODO: Implement data fetching logic
      // This should handle different query types and return structured data
      
      return {
        results: [],
        total: 0,
        hasMore: false
      };
    } catch (error) {
      this.log('error', 'Failed to fetch data', { query, error: error.message });
      throw error;
    }
  }

  async search(query, options = {}) {
    try {
      this.validateInitialized();
      
      // TODO: Implement search logic
      // This should search across the data source and return relevant results
      
      return {
        results: [],
        total: 0,
        hasMore: false
      };
    } catch (error) {
      this.log('error', 'Search failed', { query, options, error: error.message });
      throw error;
    }
  }

  async getContext(resourceId, options = {}) {
    try {
      this.validateInitialized();
      
      // TODO: Implement context retrieval logic
      // This should return detailed information about a specific resource
      
      return {
        id: resourceId,
        title: 'Resource Title',
        content: 'Resource content...',
        metadata: {},
        source: '${server.name}'
      };
    } catch (error) {
      this.log('error', 'Failed to get context', { resourceId, error: error.message });
      throw error;
    }
  }

  async cleanup() {
    try {
      // TODO: Implement cleanup logic
      // This should properly close connections, clear caches, etc.
      
      this.initialized = false;
      this.log('info', '${this.capitalizeWords(server.name)} connector cleaned up');
    } catch (error) {
      this.log('error', 'Cleanup failed', { error: error.message });
    }
  }

  // TODO: Add connector-specific helper methods
}

module.exports = ${this.toPascalCase(server.name)}Connector;
`;

    return template;
  }

  async copyAdditionalFiles(server, connectorDir) {
    // Copy any additional files that might be needed
    const filesToCopy = ['README.md', '.env.example', 'docker-compose.yml'];
    
    for (const file of filesToCopy) {
      const sourcePath = path.join(server.path, file);
      const destPath = path.join(connectorDir, file);
      
      if (await this.fileExists(sourcePath)) {
        try {
          await fs.copyFile(sourcePath, destPath);
          console.log(`📄 Copied ${file} for ${server.name}`);
        } catch (error) {
          console.warn(`⚠️  Could not copy ${file}:`, error.message);
        }
      }
    }
  }

  async generateMigrationReport() {
    const report = {
      timestamp: new Date().toISOString(),
      summary: {
        total: this.migrationLog.length,
        successful: this.migrationLog.filter(log => log.status === 'success').length,
        failed: this.migrationLog.filter(log => log.status === 'failed').length,
        skipped: this.migrationLog.filter(log => log.status === 'skipped').length
      },
      migrations: this.migrationLog
    };
    
    await fs.writeFile(
      path.join(__dirname, 'migration-report.json'),
      JSON.stringify(report, null, 2)
    );
  }

  // Utility methods
  async ensureDirectory(dirPath) {
    try {
      await fs.mkdir(dirPath, { recursive: true });
    } catch (error) {
      if (error.code !== 'EEXIST') {
        throw error;
      }
    }
  }

  async directoryExists(dirPath) {
    try {
      const stats = await fs.stat(dirPath);
      return stats.isDirectory();
    } catch {
      return false;
    }
  }

  async fileExists(filePath) {
    try {
      const stats = await fs.stat(filePath);
      return stats.isFile();
    } catch {
      return false;
    }
  }

  capitalizeWords(str) {
    return str.replace(/\b\w/g, l => l.toUpperCase());
  }

  toPascalCase(str) {
    return str
      .replace(/[-_]/g, ' ')
      .replace(/\b\w/g, l => l.toUpperCase())
      .replace(/\s/g, '');
  }
}

// Run migration if this script is executed directly
if (require.main === module) {
  const migrator = new ServerMigrator();
  migrator.migrate().catch(console.error);
}

module.exports = ServerMigrator;