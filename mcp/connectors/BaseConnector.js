import winston from 'winston';
import Joi from 'joi';

/**
 * Base Connector Interface for MCP (Model Context Protocol)
 * 
 * This interface defines the contract that all MCP connectors must implement
 * to be compatible with the unified MCP service architecture.
 */

class BaseConnector {
  constructor(config = {}) {
    this.config = config;
    this.id = config.id || this.constructor.name.toLowerCase();
    this.name = config.name || this.constructor.name;
    this.version = config.version || '1.0.0';
    this.description = config.description || '';
    this.capabilities = config.capabilities || [];
    this.isHealthy = false;
    this.lastHealthCheck = null;
    this.metadata = config.metadata || {};
  }

  /**
   * Register the connector with the MCP core service
   * @param {Object} core - The MCP core service instance
   * @returns {Promise<void>}
   */
  async register(core) {
    throw new Error('register() method must be implemented by connector');
  }

  /**
   * Initialize the connector
   * @returns {Promise<void>}
   */
  async initialize() {
    throw new Error('initialize() method must be implemented by connector');
  }

  /**
   * Health check for the connector
   * @returns {Promise<Object>} Health status object
   */
  async health() {
    try {
      const status = await this._performHealthCheck();
      this.isHealthy = status.healthy;
      this.lastHealthCheck = new Date();
      return {
        id: this.id,
        name: this.name,
        healthy: status.healthy,
        message: status.message || 'OK',
        timestamp: this.lastHealthCheck,
        metadata: status.metadata || {}
      };
    } catch (error) {
      this.isHealthy = false;
      this.lastHealthCheck = new Date();
      return {
        id: this.id,
        name: this.name,
        healthy: false,
        message: error.message,
        timestamp: this.lastHealthCheck,
        error: error.stack
      };
    }
  }

  /**
   * Get connector capabilities
   * @returns {Array<string>} List of supported context types
   */
  getCapabilities() {
    return this.capabilities;
  }

  /**
   * Get connector metadata
   * @returns {Object} Connector metadata
   */
  getMetadata() {
    return {
      id: this.id,
      name: this.name,
      version: this.version,
      description: this.description,
      capabilities: this.capabilities,
      healthy: this.isHealthy,
      lastHealthCheck: this.lastHealthCheck,
      ...this.metadata
    };
  }

  /**
   * Authenticate with the data source
   * @param {Object} credentials - Authentication credentials
   * @returns {Promise<Object>} Authentication result
   */
  async authenticate(credentials) {
    throw new Error('authenticate() method must be implemented by connector');
  }

  /**
   * Fetch data from the connector's data source
   * @param {Object} query - Query parameters
   * @returns {Promise<Object>} Fetched data
   */
  async fetch(query) {
    throw new Error('fetch() method must be implemented by connector');
  }

  /**
   * Search within the connector's data source
   * @param {Object} searchParams - Search parameters
   * @returns {Promise<Array>} Search results
   */
  async search(searchParams) {
    throw new Error('search() method must be implemented by connector');
  }

  /**
   * Get context for AI agents
   * @param {Object} contextRequest - Context request parameters
   * @returns {Promise<Object>} Context data
   */
  async getContext(contextRequest) {
    throw new Error('getContext() method must be implemented by connector');
  }

  /**
   * Cleanup resources when connector is being destroyed
   * @returns {Promise<void>}
   */
  async cleanup() {
    // Default implementation - can be overridden
    this.isHealthy = false;
  }

  /**
   * Internal health check implementation
   * @returns {Promise<Object>} Health check result
   * @protected
   */
  async _performHealthCheck() {
    // Default implementation - should be overridden by specific connectors
    return { healthy: true, message: 'Default health check passed' };
  }

  /**
   * Validate configuration
   * @returns {Object} Validation result
   * @protected
   */
  _validateConfig() {
    const errors = [];
    
    if (!this.id) {
      errors.push('Connector ID is required');
    }
    
    if (!this.name) {
      errors.push('Connector name is required');
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Public log method for connectors
   * @param {string} level - Log level (info, warn, error)
   * @param {string} message - Log message
   * @param {Object} metadata - Additional metadata
   */
  log(level, message, metadata = {}) {
    this._log(level, message, metadata);
  }

  /**
   * Log connector events
   * @param {string} level - Log level (info, warn, error)
   * @param {string} message - Log message
   * @param {Object} metadata - Additional metadata
   * @protected
   */
  _log(level, message, metadata = {}) {
    const logData = {
      connector: this.id,
      message,
      timestamp: new Date(),
      ...metadata
    };

    // This would integrate with the main logging system
    console[level](`[${this.id.toUpperCase()}] ${message}`, logData);
  }
}

export default BaseConnector;