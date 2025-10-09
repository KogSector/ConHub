/**
 * Connection Manager
 * Manages all AI agent connections, sessions, and communication logic
 */

import { EventEmitter } from 'events';
import { v4 as uuidv4 } from 'uuid';
import { McpProtocolHandler, McpMessageBuilder, ErrorCode } from '../protocols/McpProtocol.js';
import { RuleEngine } from '../rules/AgentRules.js';

/**
 * Connection States
 */
export const ConnectionState = {
  DISCONNECTED: 'disconnected',
  CONNECTING: 'connecting',
  CONNECTED: 'connected',
  AUTHENTICATING: 'authenticating',
  AUTHENTICATED: 'authenticated',
  ERROR: 'error',
  TIMEOUT: 'timeout'
};

/**
 * Agent Connection
 * Represents a connection to a specific AI agent
 */
export class AgentConnection extends EventEmitter {
  constructor(agentId, agentType, config, logger) {
    super();
    
    this.id = uuidv4();
    this.agentId = agentId;
    this.agentType = agentType;
    this.config = config;
    this.logger = logger;
    
    this.state = ConnectionState.DISCONNECTED;
    this.protocol = new McpProtocolHandler(logger);
    this.websocket = null;
    this.lastActivity = new Date();
    this.createdAt = new Date();
    
    this.heartbeatInterval = null;
    this.timeoutTimer = null;
    
    this.metrics = {
      messagesReceived: 0,
      messagesSent: 0,
      errorsCount: 0,
      toolCallsCount: 0,
      resourceReadsCount: 0
    };
    
    this.setupProtocolHandlers();
  }

  setupProtocolHandlers() {
    // Register custom handlers for this agent type
    this.protocol.registerHandler('resources/list', this.handleResourcesList.bind(this));
    this.protocol.registerHandler('resources/read', this.handleResourcesRead.bind(this));
    this.protocol.registerHandler('tools/list', this.handleToolsList.bind(this));
    this.protocol.registerHandler('tools/call', this.handleToolsCall.bind(this));
  }

  async connect(websocket) {
    this.websocket = websocket;
    this.state = ConnectionState.CONNECTING;
    
    this.logger.info(`Connecting to agent ${this.agentId}`, {
      connectionId: this.id,
      agentType: this.agentType
    });

    // Setup WebSocket event handlers
    this.websocket.on('message', this.handleMessage.bind(this));
    this.websocket.on('close', this.handleDisconnect.bind(this));
    this.websocket.on('error', this.handleError.bind(this));
    
    // Start connection timeout
    this.startConnectionTimeout();
    
    // Send initialization message
    await this.sendInitialize();
  }

  async sendInitialize() {
    const initMessage = McpMessageBuilder.createInitializeRequest(
      {
        name: 'ConHub MCP Service',
        version: '1.0.0'
      },
      {
        resources: { subscribe: true, listChanged: true },
        tools: { listChanged: true },
        prompts: { listChanged: true },
        logging: {}
      }
    );

    await this.sendMessage(initMessage);
  }

  async sendMessage(message) {
    if (!this.websocket || this.websocket.readyState !== 1) {
      throw new Error('WebSocket not connected');
    }

    const messageStr = JSON.stringify(message);
    this.websocket.send(messageStr);
    
    this.metrics.messagesSent++;
    this.lastActivity = new Date();
    
    this.logger.debug('Message sent to agent', {
      connectionId: this.id,
      agentId: this.agentId,
      method: message.method,
      id: message.id
    });
  }

  async handleMessage(data) {
    try {
      const message = JSON.parse(data.toString());
      this.metrics.messagesReceived++;
      this.lastActivity = new Date();
      
      this.logger.debug('Message received from agent', {
        connectionId: this.id,
        agentId: this.agentId,
        method: message.method,
        id: message.id
      });

      // Handle initialization response
      if (message.method === 'initialize' || (message.result && message.id === 'init')) {
        await this.handleInitializeResponse(message);
        return;
      }

      // Process message through protocol handler
      const response = await this.protocol.handleMessage(message);
      
      if (response) {
        await this.sendMessage(response);
      }

      this.emit('message', { connection: this, message, response });
      
    } catch (error) {
      this.logger.error('Error handling message from agent', {
        connectionId: this.id,
        agentId: this.agentId,
        error: error.message
      });
      
      this.metrics.errorsCount++;
      this.emit('error', { connection: this, error });
    }
  }

  async handleInitializeResponse(message) {
    if (message.error) {
      this.state = ConnectionState.ERROR;
      this.logger.error('Agent initialization failed', {
        connectionId: this.id,
        agentId: this.agentId,
        error: message.error
      });
      return;
    }

    this.state = ConnectionState.CONNECTED;
    this.clearConnectionTimeout();
    this.startHeartbeat();
    
    this.logger.info('Agent connected successfully', {
      connectionId: this.id,
      agentId: this.agentId,
      capabilities: message.result?.capabilities
    });

    this.emit('connected', { connection: this });
  }

  async handleResourcesList(message) {
    // Get resources specific to this agent type
    const resources = await this.getAgentResources();
    return { resources };
  }

  async handleResourcesRead(message) {
    const { uri } = message.params;
    
    this.metrics.resourceReadsCount++;
    
    // Validate resource access
    const validation = await this.validateResourceAccess(uri);
    if (!validation.allowed) {
      throw new Error(`Resource access denied: ${validation.reason}`);
    }

    // Read resource content
    const content = await this.readResource(uri);
    return { contents: [content] };
  }

  async handleToolsList(message) {
    // Get tools specific to this agent type
    const tools = await this.getAgentTools();
    return { tools };
  }

  async handleToolsCall(message) {
    const { name, arguments: args } = message.params;
    
    this.metrics.toolCallsCount++;
    
    // Validate tool execution
    const validation = await this.validateToolExecution(name, args);
    if (!validation.allowed) {
      throw new Error(`Tool execution denied: ${validation.reason}`);
    }

    // Execute tool
    const result = await this.executeTool(name, args);
    return { content: [{ type: 'text', text: result }] };
  }

  async getAgentResources() {
    // Override in subclasses or implement agent-specific logic
    const resourceMap = {
      'github-copilot': [
        {
          uri: 'copilot://suggestions',
          name: 'Code Suggestions',
          description: 'Real-time code suggestions and completions',
          mimeType: 'application/json'
        },
        {
          uri: 'copilot://chat',
          name: 'Copilot Chat',
          description: 'Interactive chat with GitHub Copilot',
          mimeType: 'text/plain'
        }
      ],
      'amazon-q': [
        {
          uri: 'q://code-analysis',
          name: 'Code Analysis',
          description: 'Amazon Q code analysis and recommendations',
          mimeType: 'application/json'
        },
        {
          uri: 'q://security-scan',
          name: 'Security Scan',
          description: 'Security vulnerability scanning',
          mimeType: 'application/json'
        }
      ],
      'cline': [
        {
          uri: 'cline://terminal',
          name: 'Terminal Access',
          description: 'Command line interface access',
          mimeType: 'text/plain'
        },
        {
          uri: 'cline://file-operations',
          name: 'File Operations',
          description: 'File system operations and management',
          mimeType: 'application/json'
        }
      ]
    };

    return resourceMap[this.agentType] || [];
  }

  async getAgentTools() {
    // Override in subclasses or implement agent-specific logic
    const toolMap = {
      'github-copilot': [
        {
          name: 'generate_code',
          description: 'Generate code based on natural language description',
          inputSchema: {
            type: 'object',
            properties: {
              prompt: { type: 'string' },
              language: { type: 'string' }
            },
            required: ['prompt']
          }
        },
        {
          name: 'explain_code',
          description: 'Explain existing code functionality',
          inputSchema: {
            type: 'object',
            properties: {
              code: { type: 'string' },
              language: { type: 'string' }
            },
            required: ['code']
          }
        }
      ],
      'amazon-q': [
        {
          name: 'analyze_security',
          description: 'Analyze code for security vulnerabilities',
          inputSchema: {
            type: 'object',
            properties: {
              code: { type: 'string' },
              language: { type: 'string' }
            },
            required: ['code']
          }
        },
        {
          name: 'optimize_performance',
          description: 'Suggest performance optimizations',
          inputSchema: {
            type: 'object',
            properties: {
              code: { type: 'string' },
              language: { type: 'string' }
            },
            required: ['code']
          }
        }
      ],
      'cline': [
        {
          name: 'execute_command',
          description: 'Execute shell commands',
          inputSchema: {
            type: 'object',
            properties: {
              command: { type: 'string' },
              workingDirectory: { type: 'string' }
            },
            required: ['command']
          }
        },
        {
          name: 'read_file',
          description: 'Read file contents',
          inputSchema: {
            type: 'object',
            properties: {
              path: { type: 'string' }
            },
            required: ['path']
          }
        }
      ]
    };

    return toolMap[this.agentType] || [];
  }

  async validateResourceAccess(uri) {
    // Implement resource access validation logic
    return { allowed: true };
  }

  async validateToolExecution(name, args) {
    // Implement tool execution validation logic
    return { allowed: true };
  }

  async readResource(uri) {
    // Implement resource reading logic
    return {
      uri,
      mimeType: 'application/json',
      text: JSON.stringify({
        agent: this.agentType,
        resource: uri,
        data: 'Simulated resource content',
        timestamp: new Date().toISOString()
      })
    };
  }

  async executeTool(name, args) {
    // Implement tool execution logic
    return `Executed tool ${name} with arguments: ${JSON.stringify(args)}`;
  }

  startHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }

    this.heartbeatInterval = setInterval(() => {
      this.sendHeartbeat();
    }, 30000); // 30 seconds
  }

  async sendHeartbeat() {
    try {
      const heartbeat = McpMessageBuilder.createNotification('heartbeat', {
        timestamp: new Date().toISOString()
      });
      
      await this.sendMessage(heartbeat);
    } catch (error) {
      this.logger.error('Failed to send heartbeat', {
        connectionId: this.id,
        agentId: this.agentId,
        error: error.message
      });
    }
  }

  startConnectionTimeout() {
    this.timeoutTimer = setTimeout(() => {
      if (this.state === ConnectionState.CONNECTING) {
        this.state = ConnectionState.TIMEOUT;
        this.logger.warn('Connection timeout', {
          connectionId: this.id,
          agentId: this.agentId
        });
        this.disconnect();
      }
    }, 30000); // 30 seconds
  }

  clearConnectionTimeout() {
    if (this.timeoutTimer) {
      clearTimeout(this.timeoutTimer);
      this.timeoutTimer = null;
    }
  }

  handleDisconnect() {
    this.state = ConnectionState.DISCONNECTED;
    this.cleanup();
    
    this.logger.info('Agent disconnected', {
      connectionId: this.id,
      agentId: this.agentId
    });

    this.emit('disconnected', { connection: this });
  }

  handleError(error) {
    this.state = ConnectionState.ERROR;
    this.metrics.errorsCount++;
    
    this.logger.error('WebSocket error', {
      connectionId: this.id,
      agentId: this.agentId,
      error: error.message
    });

    this.emit('error', { connection: this, error });
  }

  disconnect() {
    if (this.websocket) {
      this.websocket.close();
    }
    this.cleanup();
  }

  cleanup() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
    
    if (this.timeoutTimer) {
      clearTimeout(this.timeoutTimer);
      this.timeoutTimer = null;
    }
  }

  getStatus() {
    return {
      id: this.id,
      agentId: this.agentId,
      agentType: this.agentType,
      state: this.state,
      lastActivity: this.lastActivity,
      createdAt: this.createdAt,
      metrics: this.metrics,
      isConnected: this.state === ConnectionState.CONNECTED,
      uptime: Date.now() - this.createdAt.getTime()
    };
  }
}

/**
 * Connection Manager
 * Manages all agent connections and provides centralized connection logic
 */
export class ConnectionManager extends EventEmitter {
  constructor(logger) {
    super();
    
    this.logger = logger;
    this.connections = new Map();
    this.connectionsByAgent = new Map();
    this.ruleEngine = new RuleEngine(logger);
    
    this.metrics = {
      totalConnections: 0,
      activeConnections: 0,
      failedConnections: 0,
      totalMessages: 0,
      totalErrors: 0
    };
  }

  async createConnection(agentId, agentType, config, websocket) {
    // Validate connection request
    const currentConnections = this.getConnectionsByAgentType(agentType).length;
    const validation = await this.ruleEngine.applyRules({
      type: 'connect',
      agentType,
      connectionCount: currentConnections
    }, {});

    if (!validation.allowed) {
      throw new Error(`Connection denied: ${validation.message}`);
    }

    // Create new connection
    const connection = new AgentConnection(agentId, agentType, config, this.logger);
    
    // Setup event handlers
    connection.on('connected', this.handleConnectionEstablished.bind(this));
    connection.on('disconnected', this.handleConnectionClosed.bind(this));
    connection.on('error', this.handleConnectionError.bind(this));
    connection.on('message', this.handleConnectionMessage.bind(this));

    // Store connection
    this.connections.set(connection.id, connection);
    
    if (!this.connectionsByAgent.has(agentType)) {
      this.connectionsByAgent.set(agentType, new Set());
    }
    this.connectionsByAgent.get(agentType).add(connection.id);

    // Connect
    await connection.connect(websocket);
    
    this.metrics.totalConnections++;
    this.metrics.activeConnections++;

    this.logger.info('Connection created', {
      connectionId: connection.id,
      agentId,
      agentType
    });

    this.emit('connectionCreated', { connection });
    
    return connection;
  }

  handleConnectionEstablished(event) {
    const { connection } = event;
    
    this.logger.info('Connection established', {
      connectionId: connection.id,
      agentId: connection.agentId,
      agentType: connection.agentType
    });

    this.emit('connectionEstablished', event);
  }

  handleConnectionClosed(event) {
    const { connection } = event;
    
    // Remove from tracking
    this.connections.delete(connection.id);
    
    const agentConnections = this.connectionsByAgent.get(connection.agentType);
    if (agentConnections) {
      agentConnections.delete(connection.id);
      if (agentConnections.size === 0) {
        this.connectionsByAgent.delete(connection.agentType);
      }
    }

    this.metrics.activeConnections--;

    this.logger.info('Connection closed', {
      connectionId: connection.id,
      agentId: connection.agentId,
      agentType: connection.agentType
    });

    this.emit('connectionClosed', event);
  }

  handleConnectionError(event) {
    const { connection, error } = event;
    
    this.metrics.totalErrors++;
    
    this.logger.error('Connection error', {
      connectionId: connection.id,
      agentId: connection.agentId,
      agentType: connection.agentType,
      error: error.message
    });

    this.emit('connectionError', event);
  }

  handleConnectionMessage(event) {
    const { connection, message } = event;
    
    this.metrics.totalMessages++;
    
    this.emit('connectionMessage', event);
  }

  getConnection(connectionId) {
    return this.connections.get(connectionId);
  }

  getConnectionsByAgentType(agentType) {
    const connectionIds = this.connectionsByAgent.get(agentType) || new Set();
    return Array.from(connectionIds).map(id => this.connections.get(id)).filter(Boolean);
  }

  getConnectionsByAgent(agentId) {
    return Array.from(this.connections.values()).filter(conn => conn.agentId === agentId);
  }

  getAllConnections() {
    return Array.from(this.connections.values());
  }

  async closeConnection(connectionId) {
    const connection = this.connections.get(connectionId);
    if (connection) {
      connection.disconnect();
    }
  }

  async closeAllConnections() {
    const connections = Array.from(this.connections.values());
    await Promise.all(connections.map(conn => conn.disconnect()));
  }

  getMetrics() {
    return {
      ...this.metrics,
      connectionsByType: Object.fromEntries(
        Array.from(this.connectionsByAgent.entries()).map(([type, connections]) => [
          type,
          connections.size
        ])
      ),
      connectionDetails: this.getAllConnections().map(conn => conn.getStatus())
    };
  }

  getHealthStatus() {
    const connections = this.getAllConnections();
    const healthyConnections = connections.filter(conn => conn.state === ConnectionState.CONNECTED);
    
    return {
      status: connections.length > 0 ? 'operational' : 'idle',
      totalConnections: connections.length,
      healthyConnections: healthyConnections.length,
      unhealthyConnections: connections.length - healthyConnections.length,
      connectionTypes: Array.from(this.connectionsByAgent.keys()),
      uptime: process.uptime(),
      metrics: this.metrics
    };
  }
}
