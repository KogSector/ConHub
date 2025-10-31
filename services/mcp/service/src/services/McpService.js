import { v4 as uuidv4 } from 'uuid';
import axios from 'axios';
import { EventEmitter } from 'events';
import { ConnectorLoader } from '../connectors/loader.js';


export class McpService extends EventEmitter {
  constructor(logger) {
    super();
    this.logger = logger;
    this.connections = new Map();
    this.resources = new Map();
    this.tools = new Map();
    this.contexts = new Map();
    
    // Initialize connector loader
    this.connectorLoader = new ConnectorLoader(logger);
    this.connectors = new Map();
    
    this.protocolVersion = '2024-11-05';
    
    this.logger.info('MCP Service initialized');
    this.initializeConnectors();
  }

  // Initialize and load all connectors
  async initializeConnectors() {
    try {
      await this.connectorLoader.initialize();
      const loadedConnectors = await this.connectorLoader.loadConnectors();
      
      for (const [id, connector] of loadedConnectors) {
        this.connectors.set(id, connector);
        this.logger.info(`Connector loaded: ${id}`, { 
          name: connector.name,
          capabilities: connector.capabilities 
        });
      }
      
      this.logger.info(`Initialized ${this.connectors.size} connectors`);
    } catch (error) {
      this.logger.error('Failed to initialize connectors', { error: error.message });
    }
  }

  // Register a new connector
  async registerConnector(id, connectorInfo) {
    try {
      this.connectors.set(id, connectorInfo);
      this.logger.info(`Connector registered: ${id}`, { 
        name: connectorInfo.name,
        capabilities: connectorInfo.capabilities 
      });
      
      // Emit event for connector registration
      this.emit('connectorRegistered', { id, connector: connectorInfo });
      
      return true;
    } catch (error) {
      this.logger.error(`Failed to register connector: ${id}`, { error: error.message });
      throw error;
    }
  }

  // Get available connectors
  getConnectors() {
    return Array.from(this.connectors.entries()).map(([id, connector]) => ({
      id,
      name: connector.name,
      version: connector.version,
      capabilities: connector.capabilities,
      metadata: connector.metadata,
      status: connector.connector ? 'active' : 'inactive'
    }));
  }

  // Get specific connector
  getConnector(id) {
    return this.connectors.get(id);
  }

  
  async initializeConnection(agentId, config) {
    try {
      const connectionId = uuidv4();
      
      const connection = {
        id: connectionId,
        agentId,
        config,
        status: 'connecting',
        capabilities: null,
        serverInfo: null,
        createdAt: new Date(),
        lastActivity: new Date()
      };

      this.connections.set(connectionId, connection);
      
      
      const initResponse = await this.sendMcpMessage(connectionId, {
        jsonrpc: '2.0',
        id: uuidv4(),
        method: 'initialize',
        params: {
          protocolVersion: this.protocolVersion,
          capabilities: {
            resources: { subscribe: true, listChanged: true },
            tools: { listChanged: true },
            prompts: { listChanged: true },
            logging: {}
          },
          clientInfo: {
            name: 'ConHub AI Agents Service',
            version: '1.0.0'
          }
        }
      });

      if (initResponse && initResponse.result) {
        connection.status = 'connected';
        connection.capabilities = initResponse.result.capabilities;
        connection.serverInfo = initResponse.result.serverInfo;
        
        this.logger.info('MCP connection initialized', { 
          connectionId, 
          agentId,
          serverInfo: connection.serverInfo 
        });
        
        this.emit('connectionEstablished', connection);
        return connection;
      } else {
        throw new Error('Failed to initialize MCP connection');
      }
    } catch (error) {
      this.logger.error('MCP connection initialization failed', { agentId, error: error.message });
      throw error;
    }
  }

  
  async listResources(connectionId) {
    try {
      const response = await this.sendMcpMessage(connectionId, {
        jsonrpc: '2.0',
        id: uuidv4(),
        method: 'resources/list',
        params: {}
      });

      if (response && response.result && response.result.resources) {
        const resources = response.result.resources;
        
        
        resources.forEach(resource => {
          this.resources.set(resource.uri, {
            ...resource,
            connectionId,
            lastUpdated: new Date()
          });
        });

        this.logger.info('Resources listed', { connectionId, count: resources.length });
        return resources;
      }
      
      return [];
    } catch (error) {
      this.logger.error('Failed to list resources', { connectionId, error: error.message });
      throw error;
    }
  }

  
  async readResource(connectionId, uri) {
    try {
      const response = await this.sendMcpMessage(connectionId, {
        jsonrpc: '2.0',
        id: uuidv4(),
        method: 'resources/read',
        params: { uri }
      });

      if (response && response.result) {
        this.logger.info('Resource read', { connectionId, uri });
        return response.result;
      }
      
      throw new Error('Failed to read resource');
    } catch (error) {
      this.logger.error('Failed to read resource', { connectionId, uri, error: error.message });
      throw error;
    }
  }

  
  async listTools(connectionId) {
    try {
      const response = await this.sendMcpMessage(connectionId, {
        jsonrpc: '2.0',
        id: uuidv4(),
        method: 'tools/list',
        params: {}
      });

      if (response && response.result && response.result.tools) {
        const tools = response.result.tools;
        
        
        tools.forEach(tool => {
          this.tools.set(`${connectionId}:${tool.name}`, {
            ...tool,
            connectionId,
            lastUpdated: new Date()
          });
        });

        this.logger.info('Tools listed', { connectionId, count: tools.length });
        return tools;
      }
      
      return [];
    } catch (error) {
      this.logger.error('Failed to list tools', { connectionId, error: error.message });
      throw error;
    }
  }

  
  async callTool(connectionId, toolName, arguments_) {
    try {
      const response = await this.sendMcpMessage(connectionId, {
        jsonrpc: '2.0',
        id: uuidv4(),
        method: 'tools/call',
        params: {
          name: toolName,
          arguments: arguments_
        }
      });

      if (response && response.result) {
        this.logger.info('Tool called successfully', { connectionId, toolName });
        return response.result;
      }
      
      throw new Error('Tool call failed');
    } catch (error) {
      this.logger.error('Failed to call tool', { connectionId, toolName, error: error.message });
      throw error;
    }
  }

  
  async subscribeToResource(connectionId, uri) {
    try {
      const response = await this.sendMcpMessage(connectionId, {
        jsonrpc: '2.0',
        id: uuidv4(),
        method: 'resources/subscribe',
        params: { uri }
      });

      if (response && response.result) {
        this.logger.info('Subscribed to resource', { connectionId, uri });
        return true;
      }
      
      return false;
    } catch (error) {
      this.logger.error('Failed to subscribe to resource', { connectionId, uri, error: error.message });
      throw error;
    }
  }

  
  async sendMcpMessage(connectionId, message) {
    const connection = this.connections.get(connectionId);
    if (!connection) {
      throw new Error(`Connection ${connectionId} not found`);
    }

    try {
      
      connection.lastActivity = new Date();
      
      // Route to external MCP servers based on agentId, URI, or tool name
      const response = await this.routeToExternalMcp(connection, message);
      
      this.logger.debug('MCP message sent', { connectionId, method: message.method });
      return response;
    } catch (error) {
      this.logger.error('Failed to send MCP message', { connectionId, error: error.message });
      throw error;
    }
  }

  /**
   * Route MCP requests to appropriate external MCP server
   * Routes based on URI prefixes, tool names, or agentId
   */
  async routeToExternalMcp(connection, message) {
    try {
      const targetEndpoint = this.determineTargetEndpoint(connection, message);
      
      if (!targetEndpoint) {
        // No routing needed - handle locally (e.g., initialize)
        return this.handleLocalMethod(connection, message);
      }

      this.logger.debug('Routing MCP request to external server', {
        endpoint: targetEndpoint,
        method: message.method
      });

      // Forward request to external MCP server
      const response = await axios.post(targetEndpoint, message, {
        headers: {
          'Content-Type': 'application/json',
        },
        timeout: 30000, // 30 second timeout
      });

      return response.data;
    } catch (error) {
      this.logger.error('External MCP request failed', {
        error: error.message,
        method: message.method
      });

      // Return JSON-RPC error response
      return {
        jsonrpc: '2.0',
        id: message.id,
        error: {
          code: -32603,
          message: `External MCP server error: ${error.message}`,
          data: error.response?.data || null
        }
      };
    }
  }

  /**
   * Determine which external MCP server to route the request to
   * Returns endpoint URL or null for local handling
   */
  determineTargetEndpoint(connection, message) {
    const { method, params } = message;
    
    // Check URI-based routing for resources
    if (params && params.uri) {
      const uri = params.uri;
      
      if (uri.startsWith('gdrive://') || uri.startsWith('google-drive://')) {
        return process.env.MCP_GOOGLE_DRIVE_ENDPOINT || 'http://localhost:3005';
      }
      
      if (uri.startsWith('dropbox://')) {
        return process.env.MCP_DROPBOX_ENDPOINT || 'http://localhost:3006';
      }
      
      if (uri.startsWith('file://')) {
        return process.env.MCP_FILESYSTEM_ENDPOINT || 'http://localhost:3007';
      }
    }

    // Check tool name-based routing
    if (params && params.name) {
      const toolName = params.name.toLowerCase();
      
      if (toolName.includes('drive') || toolName.includes('gdrive')) {
        return process.env.MCP_GOOGLE_DRIVE_ENDPOINT || 'http://localhost:3005';
      }
      
      if (toolName.includes('dropbox')) {
        return process.env.MCP_DROPBOX_ENDPOINT || 'http://localhost:3006';
      }
      
      if (toolName.includes('file') || toolName.includes('filesystem')) {
        return process.env.MCP_FILESYSTEM_ENDPOINT || 'http://localhost:3007';
      }
    }

    // Check agentId-based routing for backwards compatibility
    if (connection.agentId) {
      const agentId = connection.agentId.toLowerCase();
      
      if (agentId.includes('drive') || agentId.includes('gdrive')) {
        return process.env.MCP_GOOGLE_DRIVE_ENDPOINT || 'http://localhost:3005';
      }
      
      if (agentId.includes('dropbox')) {
        return process.env.MCP_DROPBOX_ENDPOINT || 'http://localhost:3006';
      }
      
      if (agentId.includes('file') || agentId.includes('filesystem')) {
        return process.env.MCP_FILESYSTEM_ENDPOINT || 'http://localhost:3007';
      }
    }

    // Route all resources/tools list requests to filesystem by default
    // This provides basic file operations as a fallback
    if (method === 'resources/list' || method === 'tools/list') {
      return process.env.MCP_FILESYSTEM_ENDPOINT || 'http://localhost:3007';
    }

    // No external routing - handle locally
    return null;
  }

  /**
   * Handle methods that don't need external MCP servers
   * (e.g., initialize, connection management)
   */
  handleLocalMethod(connection, message) {
    const { method } = message;

    switch (method) {
      case 'initialize':
        return {
          jsonrpc: '2.0',
          id: message.id,
          result: {
            protocolVersion: this.protocolVersion,
            capabilities: {
              resources: { subscribe: true, listChanged: true },
              tools: { listChanged: true },
              prompts: { listChanged: true }
            },
            serverInfo: {
              name: `ConHub MCP Proxy`,
              version: '1.0.0'
            }
          }
        };

      default:
        return {
          jsonrpc: '2.0',
          id: message.id,
          error: {
            code: -32601,
            message: 'Method not found or no route available'
          }
        };
    }
  }

  
  async closeConnection(connectionId) {
    const connection = this.connections.get(connectionId);
    if (connection) {
      connection.status = 'disconnected';
      this.connections.delete(connectionId);
      
      this.logger.info('MCP connection closed', { connectionId });
      this.emit('connectionClosed', connection);
    }
  }

  
  getConnections() {
    return Array.from(this.connections.values());
  }

  
  getConnection(connectionId) {
    return this.connections.get(connectionId);
  }

  
  getHealthStatus() {
    const connections = this.getConnections();
    const activeConnections = connections.filter(c => c.status === 'connected');
    
    return {
      status: 'healthy',
      connections: {
        total: connections.length,
        active: activeConnections.length,
        inactive: connections.length - activeConnections.length
      },
      resources: this.resources.size,
      tools: this.tools.size,
      uptime: process.uptime()
    };
  }

  // Connector-related methods
  async searchConnectors(query, options = {}) {
    try {
      const results = [];
      
      for (const [id, connectorInfo] of this.connectors) {
        if (connectorInfo.connector && typeof connectorInfo.connector.search === 'function') {
          try {
            const connectorResults = await connectorInfo.connector.search(query, options);
            results.push({
              connectorId: id,
              connectorName: connectorInfo.name,
              ...connectorResults
            });
          } catch (error) {
            this.logger.warn(`Search failed for connector ${id}`, { error: error.message });
          }
        }
      }
      
      return {
        query,
        results,
        totalConnectors: this.connectors.size,
        searchedConnectors: results.length
      };
    } catch (error) {
      this.logger.error('Connector search failed', { error: error.message });
      throw error;
    }
  }

  async getConnectorContext(connectorId, resourceId, options = {}) {
    try {
      const connectorInfo = this.connectors.get(connectorId);
      
      if (!connectorInfo || !connectorInfo.connector) {
        throw new Error(`Connector not found: ${connectorId}`);
      }

      if (typeof connectorInfo.connector.getContext !== 'function') {
        throw new Error(`Connector ${connectorId} does not support context retrieval`);
      }

      return await connectorInfo.connector.getContext(resourceId, options);
    } catch (error) {
      this.logger.error('Failed to get connector context', { 
        connectorId, 
        resourceId, 
        error: error.message 
      });
      throw error;
    }
  }

  async fetchConnectorData(connectorId, query) {
    try {
      const connectorInfo = this.connectors.get(connectorId);
      
      if (!connectorInfo || !connectorInfo.connector) {
        throw new Error(`Connector not found: ${connectorId}`);
      }

      if (typeof connectorInfo.connector.fetchData !== 'function') {
        throw new Error(`Connector ${connectorId} does not support data fetching`);
      }

      return await connectorInfo.connector.fetchData(query);
    } catch (error) {
      this.logger.error('Failed to fetch connector data', { 
        connectorId, 
        query, 
        error: error.message 
      });
      throw error;
    }
  }

  async getConnectorHealth(connectorId) {
    try {
      const connectorInfo = this.connectors.get(connectorId);
      
      if (!connectorInfo || !connectorInfo.connector) {
        return {
          status: 'not_found',
          message: `Connector not found: ${connectorId}`
        };
      }

      if (typeof connectorInfo.connector.healthCheck === 'function') {
        return await connectorInfo.connector.healthCheck();
      }

      return {
        status: 'unknown',
        message: 'Health check not supported'
      };
    } catch (error) {
      this.logger.error('Connector health check failed', { 
        connectorId, 
        error: error.message 
      });
      return {
        status: 'error',
        message: error.message
      };
    }
  }

  async getAllConnectorHealth() {
    const healthStatus = {};
    
    for (const [id, connectorInfo] of this.connectors) {
      healthStatus[id] = await this.getConnectorHealth(id);
    }
    
    return healthStatus;
  }

  // Cleanup connectors on service shutdown
  async cleanup() {
    try {
      this.logger.info('Cleaning up MCP Service and connectors');
      
      for (const [id, connectorInfo] of this.connectors) {
        if (connectorInfo.connector && typeof connectorInfo.connector.cleanup === 'function') {
          try {
            await connectorInfo.connector.cleanup();
            this.logger.info(`Connector ${id} cleaned up successfully`);
          } catch (error) {
            this.logger.warn(`Failed to cleanup connector ${id}`, { error: error.message });
          }
        }
      }
      
      if (this.connectorLoader) {
        await this.connectorLoader.cleanup();
      }
      
      this.connectors.clear();
      this.connections.clear();
      this.resources.clear();
      this.tools.clear();
      this.contexts.clear();
      
      this.logger.info('MCP Service cleanup completed');
    } catch (error) {
      this.logger.error('MCP Service cleanup failed', { error: error.message });
    }
  }
}
