import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import winston from 'winston';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Connector Loader for MCP Service
 * 
 * Dynamically loads and manages MCP connectors from various sources:
 * - In-process modules from connectors/ directory
 * - External connector containers via HTTP/gRPC
 * - Plugin-based connectors
 */
export class ConnectorLoader {
  constructor(config = {}) {
    this.config = config;
    this.connectors = new Map();
    this.connectorsPath = config.connectorsPath || path.resolve(__dirname, '../../../connectors');
    this.logger = config.logger || winston.createLogger({
      level: 'info',
      format: winston.format.simple(),
      transports: [new winston.transports.Console()]
    });
    this.registry = new Map();
    this.healthCheckInterval = config.healthCheckInterval || 30000; // 30 seconds
    this.healthCheckTimer = null;
  }

  /**
   * Initialize the connector loader
   * @returns {Promise<void>}
   */
  async initialize() {
    this.logger.info('Initializing Connector Loader...');
    
    try {
      // Load in-process connectors
      await this.loadInProcessConnectors();
      
      // Load external connector configurations
      await this.loadExternalConnectors();
      
      // Start health monitoring
      this.startHealthMonitoring();
      
      this.logger.info(`Connector Loader initialized with ${this.connectors.size} connectors`);
    } catch (error) {
      this.logger.error('Failed to initialize Connector Loader:', error);
      throw error;
    }
  }

  /**
   * Load in-process connectors from the connectors directory
   * @returns {Promise<void>}
   */
  async loadInProcessConnectors() {
    if (!fs.existsSync(this.connectorsPath)) {
      this.logger.warn(`Connectors directory not found: ${this.connectorsPath}`);
      return;
    }

    const connectorDirs = fs.readdirSync(this.connectorsPath, { withFileTypes: true })
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name);

    for (const connectorDir of connectorDirs) {
      try {
        await this.loadConnector(connectorDir);
      } catch (error) {
        this.logger.error(`Failed to load connector ${connectorDir}:`, error);
      }
    }
  }

  /**
   * Load a specific connector
   * @param {string} connectorName - Name of the connector to load
   * @returns {Promise<void>}
   */
  async loadConnector(connectorName) {
    const connectorPath = path.join(this.connectorsPath, connectorName);
    const indexPath = path.join(connectorPath, 'index.js');
    
    if (!fs.existsSync(indexPath)) {
      throw new Error(`Connector index.js not found: ${indexPath}`);
    }

    try {
      // Dynamic import of the connector module
      const connectorModule = await import(`file://${indexPath}`);
      const ConnectorClass = connectorModule.default || connectorModule[connectorName];
      
      if (!ConnectorClass) {
        throw new Error(`Connector class not found in ${indexPath}`);
      }

      // Load connector configuration
      const configPath = path.join(connectorPath, 'config.json');
      let connectorConfig = {};
      
      if (fs.existsSync(configPath)) {
        connectorConfig = JSON.parse(fs.readFileSync(configPath, 'utf8'));
      }

      // Create connector instance
      const connector = new ConnectorClass(connectorConfig);
      
      // Validate connector implements required interface
      this.validateConnector(connector);
      
      // Initialize the connector
      await connector.initialize();
      
      // Register the connector
      this.connectors.set(connector.id, connector);
      this.registry.set(connector.id, {
        type: 'in-process',
        instance: connector,
        metadata: connector.getMetadata(),
        loadedAt: new Date()
      });

      this.logger.info(`Loaded in-process connector: ${connector.id}`);
    } catch (error) {
      this.logger.error(`Failed to load connector ${connectorName}:`, error);
      throw error;
    }
  }

  /**
   * Load external connector configurations
   * @returns {Promise<void>}
   */
  async loadExternalConnectors() {
    const externalConfig = this.config.externalConnectors || [];
    
    for (const connectorConfig of externalConfig) {
      try {
        await this.registerExternalConnector(connectorConfig);
      } catch (error) {
        this.logger.error(`Failed to register external connector ${connectorConfig.id}:`, error);
      }
    }
  }

  /**
   * Register an external connector (container or service)
   * @param {Object} config - External connector configuration
   * @returns {Promise<void>}
   */
  async registerExternalConnector(config) {
    const { id, name, endpoint, type = 'http', healthEndpoint } = config;
    
    if (!id || !endpoint) {
      throw new Error('External connector must have id and endpoint');
    }

    // Create a proxy connector for external services
    const proxyConnector = {
      id,
      name: name || id,
      type: 'external',
      endpoint,
      healthEndpoint: healthEndpoint || `${endpoint}/health`,
      config,
      
      async health() {
        try {
          const response = await fetch(this.healthEndpoint);
          const data = await response.json();
          return {
            id: this.id,
            healthy: response.ok && data.healthy,
            message: data.message || 'External connector health check',
            timestamp: new Date()
          };
        } catch (error) {
          return {
            id: this.id,
            healthy: false,
            message: error.message,
            timestamp: new Date()
          };
        }
      },
      
      async fetch(query) {
        const response = await fetch(`${this.endpoint}/fetch`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(query)
        });
        return response.json();
      },
      
      async search(searchParams) {
        const response = await fetch(`${this.endpoint}/search`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(searchParams)
        });
        return response.json();
      },
      
      async getContext(contextRequest) {
        const response = await fetch(`${this.endpoint}/context`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(contextRequest)
        });
        return response.json();
      }
    };

    this.connectors.set(id, proxyConnector);
    this.registry.set(id, {
      type: 'external',
      instance: proxyConnector,
      metadata: config,
      loadedAt: new Date()
    });

    this.logger.info(`Registered external connector: ${id} at ${endpoint}`);
  }

  /**
   * Validate that a connector implements the required interface
   * @param {Object} connector - Connector instance to validate
   */
  validateConnector(connector) {
    const requiredMethods = ['initialize', 'health', 'fetch', 'search', 'getContext'];
    
    for (const method of requiredMethods) {
      if (typeof connector[method] !== 'function') {
        throw new Error(`Connector ${connector.id} missing required method: ${method}`);
      }
    }
    
    if (!connector.id || !connector.name) {
      throw new Error(`Connector must have id and name properties`);
    }
  }

  /**
   * Get a connector by ID
   * @param {string} connectorId - Connector ID
   * @returns {Object|null} Connector instance or null if not found
   */
  getConnector(connectorId) {
    return this.connectors.get(connectorId) || null;
  }

  /**
   * Get all registered connectors
   * @returns {Map} Map of all connectors
   */
  getAllConnectors() {
    return new Map(this.connectors);
  }

  /**
   * Get connector registry information
   * @returns {Array} Array of connector registry entries
   */
  getRegistry() {
    return Array.from(this.registry.values());
  }

  /**
   * Get health status of all connectors
   * @returns {Promise<Array>} Array of health status objects
   */
  async getHealthStatus() {
    const healthPromises = Array.from(this.connectors.values()).map(async (connector) => {
      try {
        return await connector.health();
      } catch (error) {
        return {
          id: connector.id,
          healthy: false,
          message: error.message,
          timestamp: new Date()
        };
      }
    });

    return Promise.all(healthPromises);
  }

  /**
   * Start health monitoring for all connectors
   */
  startHealthMonitoring() {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
    }

    this.healthCheckTimer = setInterval(async () => {
      try {
        const healthStatuses = await this.getHealthStatus();
        const unhealthyConnectors = healthStatuses.filter(status => !status.healthy);
        
        if (unhealthyConnectors.length > 0) {
          this.logger.warn(`Unhealthy connectors detected:`, unhealthyConnectors.map(c => c.id));
        }
      } catch (error) {
        this.logger.error('Health monitoring error:', error);
      }
    }, this.healthCheckInterval);

    this.logger.info(`Started health monitoring with ${this.healthCheckInterval}ms interval`);
  }

  /**
   * Stop health monitoring
   */
  stopHealthMonitoring() {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
      this.healthCheckTimer = null;
      this.logger.info('Stopped health monitoring');
    }
  }

  /**
   * Reload a specific connector
   * @param {string} connectorId - Connector ID to reload
   * @returns {Promise<void>}
   */
  async reloadConnector(connectorId) {
    const registryEntry = this.registry.get(connectorId);
    
    if (!registryEntry) {
      throw new Error(`Connector ${connectorId} not found in registry`);
    }

    if (registryEntry.type === 'in-process') {
      // Cleanup existing connector
      const existingConnector = this.connectors.get(connectorId);
      if (existingConnector && typeof existingConnector.cleanup === 'function') {
        await existingConnector.cleanup();
      }

      // Remove from maps
      this.connectors.delete(connectorId);
      this.registry.delete(connectorId);

      // Reload the connector
      await this.loadConnector(connectorId);
      
      this.logger.info(`Reloaded connector: ${connectorId}`);
    } else {
      this.logger.warn(`Cannot reload external connector: ${connectorId}`);
    }
  }

  /**
   * Get health status of all connectors
   * @returns {Promise<Array>} Array of health check results
   */
  async getHealth() {
    const healthResults = [];
    
    for (const [id, connector] of this.connectors) {
      try {
        if (typeof connector.health === 'function') {
          const health = await connector.health();
          healthResults.push(health);
        } else {
          healthResults.push({
            id,
            healthy: true,
            message: 'Health check not implemented',
            timestamp: new Date()
          });
        }
      } catch (error) {
        healthResults.push({
          id,
          healthy: false,
          message: error.message,
          timestamp: new Date()
        });
      }
    }
    
    return healthResults;
  }

  /**
   * Discover available connectors in the connectors directory
   * @returns {Promise<Array>} Array of connector metadata
   */
  async discoverConnectors() {
    const discovered = [];
    
    if (!fs.existsSync(this.connectorsPath)) {
      this.logger.warn(`Connectors directory not found: ${this.connectorsPath}`);
      return discovered;
    }

    const connectorDirs = fs.readdirSync(this.connectorsPath, { withFileTypes: true })
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name);

    for (const connectorDir of connectorDirs) {
      try {
        const connectorPath = path.join(this.connectorsPath, connectorDir);
        const configPath = path.join(connectorPath, 'config.json');
        const indexPath = path.join(connectorPath, 'index.js');
        
        if (fs.existsSync(configPath) && fs.existsSync(indexPath)) {
          const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
          discovered.push({
            id: config.id || connectorDir,
            name: config.name || connectorDir,
            version: config.version || '1.0.0',
            description: config.description || '',
            capabilities: config.capabilities || [],
            path: connectorPath,
            loaded: this.connectors.has(config.id || connectorDir)
          });
        }
      } catch (error) {
        this.logger.error(`Failed to discover connector ${connectorDir}:`, error);
      }
    }
    
    return discovered;
  }

  /**
   * Cleanup all connectors and stop monitoring
   * @returns {Promise<void>}
   */
  async cleanup() {
    this.stopHealthMonitoring();

    const cleanupPromises = Array.from(this.connectors.values()).map(async (connector) => {
      if (typeof connector.cleanup === 'function') {
        try {
          await connector.cleanup();
        } catch (error) {
          this.logger.error(`Error cleaning up connector ${connector.id}:`, error);
        }
      }
    });

    await Promise.all(cleanupPromises);
    
    this.connectors.clear();
    this.registry.clear();
    
    this.logger.info('Connector Loader cleanup completed');
  }
}

export default ConnectorLoader;