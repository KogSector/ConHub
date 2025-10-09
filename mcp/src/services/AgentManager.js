import { v4 as uuidv4 } from 'uuid';
import { EventEmitter } from 'events';

/**
 * Agent Manager
 * Manages AI agent connections, sessions, and orchestrates MCP and webhook services
 */
export class AgentManager extends EventEmitter {
  constructor(logger, mcpService, webhookService) {
    super();
    this.logger = logger;
    this.mcpService = mcpService;
    this.webhookService = webhookService;
    
    this.agents = new Map();
    this.sessions = new Map();
    this.webSocketConnections = new Map();
    
    // Agent configurations
    this.agentConfigs = {
      'github-copilot': {
        name: 'GitHub Copilot',
        type: 'code-assistant',
        capabilities: ['code-completion', 'code-explanation', 'chat'],
        endpoints: {
          api: process.env.GITHUB_COPILOT_API_URL,
          webhook: '/api/webhooks/github-copilot'
        },
        authentication: {
          type: 'oauth',
          tokenUrl: 'https://github.com/login/oauth/access_token'
        }
      },
      'amazon-q': {
        name: 'Amazon Q',
        type: 'code-assistant',
        capabilities: ['code-analysis', 'security-scan', 'optimization', 'chat'],
        endpoints: {
          api: process.env.AMAZON_Q_API_URL,
          webhook: '/api/webhooks/amazon-q'
        },
        authentication: {
          type: 'aws-iam',
          region: process.env.AWS_REGION || 'us-east-1'
        }
      },
      'cline': {
        name: 'Cline',
        type: 'terminal-assistant',
        capabilities: ['command-execution', 'file-operations', 'task-automation'],
        endpoints: {
          api: process.env.CLINE_API_URL,
          webhook: '/api/webhooks/cline'
        },
        authentication: {
          type: 'api-key',
          keyHeader: 'X-Cline-API-Key'
        }
      }
    };
    
    this.setupEventListeners();
    this.logger.info('Agent Manager initialized');
  }

  /**
   * Setup event listeners for MCP and webhook services
   */
  setupEventListeners() {
    // MCP service events
    this.mcpService.on('connectionEstablished', (connection) => {
      this.handleMcpConnectionEstablished(connection);
    });

    this.mcpService.on('connectionClosed', (connection) => {
      this.handleMcpConnectionClosed(connection);
    });

    // Webhook service events
    this.webhookService.on('webhookReceived', (webhookData) => {
      this.handleWebhookReceived(webhookData);
    });

    // Specific agent events
    this.webhookService.on('copilotUsage', (data) => {
      this.handleCopilotUsage(data);
    });

    this.webhookService.on('codeAnalysisComplete', (data) => {
      this.handleCodeAnalysisComplete(data);
    });

    this.webhookService.on('commandExecuted', (data) => {
      this.handleCommandExecuted(data);
    });
  }

  /**
   * Register a new AI agent
   */
  async registerAgent(agentId, config = null) {
    try {
      const agentConfig = config || this.agentConfigs[agentId];
      if (!agentConfig) {
        throw new Error(`No configuration found for agent: ${agentId}`);
      }

      const agent = {
        id: agentId,
        config: agentConfig,
        status: 'registered',
        mcpConnection: null,
        sessions: new Set(),
        capabilities: agentConfig.capabilities,
        registeredAt: new Date(),
        lastActivity: new Date(),
        metrics: {
          totalSessions: 0,
          totalRequests: 0,
          totalWebhooks: 0,
          errors: 0
        }
      };

      this.agents.set(agentId, agent);
      
      this.logger.info('Agent registered', { 
        agentId, 
        capabilities: agent.capabilities 
      });

      this.emit('agentRegistered', agent);
      return agent;
    } catch (error) {
      this.logger.error('Failed to register agent', { agentId, error: error.message });
      throw error;
    }
  }

  /**
   * Connect to an AI agent via MCP
   */
  async connectAgent(agentId, connectionConfig = {}) {
    try {
      const agent = this.agents.get(agentId);
      if (!agent) {
        throw new Error(`Agent ${agentId} not registered`);
      }

      // Initialize MCP connection
      const mcpConnection = await this.mcpService.initializeConnection(agentId, {
        ...agent.config,
        ...connectionConfig
      });

      agent.mcpConnection = mcpConnection;
      agent.status = 'connected';
      agent.lastActivity = new Date();

      this.logger.info('Agent connected via MCP', { agentId });
      this.emit('agentConnected', agent);

      return agent;
    } catch (error) {
      this.logger.error('Failed to connect agent', { agentId, error: error.message });
      
      const agent = this.agents.get(agentId);
      if (agent) {
        agent.status = 'error';
        agent.metrics.errors++;
      }
      
      throw error;
    }
  }

  /**
   * Create a new session with an AI agent
   */
  async createSession(agentId, userId, sessionConfig = {}) {
    try {
      const agent = this.agents.get(agentId);
      if (!agent) {
        throw new Error(`Agent ${agentId} not found`);
      }

      const sessionId = uuidv4();
      const session = {
        id: sessionId,
        agentId,
        userId,
        config: sessionConfig,
        status: 'active',
        createdAt: new Date(),
        lastActivity: new Date(),
        messages: [],
        context: {},
        metrics: {
          messageCount: 0,
          toolCalls: 0,
          errors: 0
        }
      };

      this.sessions.set(sessionId, session);
      agent.sessions.add(sessionId);
      agent.metrics.totalSessions++;
      agent.lastActivity = new Date();

      this.logger.info('Session created', { sessionId, agentId, userId });
      this.emit('sessionCreated', session);

      return session;
    } catch (error) {
      this.logger.error('Failed to create session', { agentId, userId, error: error.message });
      throw error;
    }
  }

  /**
   * Send message to AI agent via MCP
   */
  async sendMessage(sessionId, message, options = {}) {
    try {
      const session = this.sessions.get(sessionId);
      if (!session) {
        throw new Error(`Session ${sessionId} not found`);
      }

      const agent = this.agents.get(session.agentId);
      if (!agent || !agent.mcpConnection) {
        throw new Error(`Agent ${session.agentId} not connected`);
      }

      // Add message to session
      const messageObj = {
        id: uuidv4(),
        type: 'user',
        content: message,
        timestamp: new Date(),
        options
      };

      session.messages.push(messageObj);
      session.metrics.messageCount++;
      session.lastActivity = new Date();
      agent.lastActivity = new Date();
      agent.metrics.totalRequests++;

      // Process message based on agent capabilities
      let response;
      if (agent.capabilities.includes('chat')) {
        response = await this.handleChatMessage(agent, session, messageObj);
      } else if (agent.capabilities.includes('code-completion')) {
        response = await this.handleCodeCompletion(agent, session, messageObj);
      } else {
        response = await this.handleGenericMessage(agent, session, messageObj);
      }

      // Add response to session
      const responseObj = {
        id: uuidv4(),
        type: 'assistant',
        content: response,
        timestamp: new Date()
      };

      session.messages.push(responseObj);

      this.logger.info('Message processed', { sessionId, messageId: messageObj.id });
      this.emit('messageProcessed', { session, message: messageObj, response: responseObj });

      return responseObj;
    } catch (error) {
      this.logger.error('Failed to send message', { sessionId, error: error.message });
      
      const session = this.sessions.get(sessionId);
      if (session) {
        session.metrics.errors++;
        const agent = this.agents.get(session.agentId);
        if (agent) {
          agent.metrics.errors++;
        }
      }
      
      throw error;
    }
  }

  /**
   * Handle chat messages
   */
  async handleChatMessage(agent, session, message) {
    const tools = await this.mcpService.listTools(agent.mcpConnection.id);
    
    // Find appropriate tool for chat
    const chatTool = tools.find(tool => 
      tool.name.includes('chat') || 
      tool.name.includes('conversation') ||
      tool.name.includes('generate')
    );

    if (chatTool) {
      const result = await this.mcpService.callTool(
        agent.mcpConnection.id,
        chatTool.name,
        { prompt: message.content, context: session.context }
      );
      
      return result.content?.[0]?.text || 'No response generated';
    }

    return `Chat response from ${agent.config.name}: ${message.content}`;
  }

  /**
   * Handle code completion requests
   */
  async handleCodeCompletion(agent, session, message) {
    const tools = await this.mcpService.listTools(agent.mcpConnection.id);
    
    const completionTool = tools.find(tool => 
      tool.name.includes('complete') || 
      tool.name.includes('generate') ||
      tool.name.includes('suggest')
    );

    if (completionTool) {
      const result = await this.mcpService.callTool(
        agent.mcpConnection.id,
        completionTool.name,
        { 
          code: message.content, 
          language: message.options.language || 'javascript'
        }
      );
      
      return result.content?.[0]?.text || 'No completion generated';
    }

    return `Code completion from ${agent.config.name} for: ${message.content}`;
  }

  /**
   * Handle generic messages
   */
  async handleGenericMessage(agent, session, message) {
    const tools = await this.mcpService.listTools(agent.mcpConnection.id);
    
    if (tools.length > 0) {
      // Use the first available tool
      const tool = tools[0];
      const result = await this.mcpService.callTool(
        agent.mcpConnection.id,
        tool.name,
        { input: message.content }
      );
      
      return result.content?.[0]?.text || 'No response generated';
    }

    return `Response from ${agent.config.name}: Processed "${message.content}"`;
  }

  /**
   * Handle WebSocket messages
   */
  async handleWebSocketMessage(ws, data) {
    try {
      const { type, payload } = data;

      switch (type) {
        case 'register_agent':
          await this.handleWebSocketAgentRegistration(ws, payload);
          break;

        case 'create_session':
          await this.handleWebSocketSessionCreation(ws, payload);
          break;

        case 'send_message':
          await this.handleWebSocketMessage(ws, payload);
          break;

        case 'subscribe_events':
          await this.handleWebSocketEventSubscription(ws, payload);
          break;

        default:
          ws.send(JSON.stringify({
            type: 'error',
            message: `Unknown message type: ${type}`
          }));
      }
    } catch (error) {
      this.logger.error('WebSocket message handling error', { error: error.message });
      ws.send(JSON.stringify({
        type: 'error',
        message: error.message
      }));
    }
  }

  /**
   * Handle WebSocket disconnection
   */
  handleWebSocketDisconnection(ws) {
    // Clean up WebSocket connection data
    for (const [connectionId, connection] of this.webSocketConnections.entries()) {
      if (connection.ws === ws) {
        this.webSocketConnections.delete(connectionId);
        this.logger.info('WebSocket connection cleaned up', { connectionId });
        break;
      }
    }
  }

  /**
   * Handle MCP connection established
   */
  handleMcpConnectionEstablished(connection) {
    const agent = this.agents.get(connection.agentId);
    if (agent) {
      agent.status = 'connected';
      agent.mcpConnection = connection;
      this.emit('agentConnected', agent);
    }
  }

  /**
   * Handle MCP connection closed
   */
  handleMcpConnectionClosed(connection) {
    const agent = this.agents.get(connection.agentId);
    if (agent) {
      agent.status = 'disconnected';
      agent.mcpConnection = null;
      this.emit('agentDisconnected', agent);
    }
  }

  /**
   * Handle webhook received
   */
  handleWebhookReceived(webhookData) {
    const agent = this.agents.get(webhookData.agentType);
    if (agent) {
      agent.metrics.totalWebhooks++;
      agent.lastActivity = new Date();
      this.emit('agentWebhookReceived', { agent, webhookData });
    }
  }

  /**
   * Handle Copilot usage events
   */
  handleCopilotUsage(data) {
    this.logger.info('Copilot usage tracked', { user: data.user?.login });
    this.emit('copilotUsageTracked', data);
  }

  /**
   * Handle code analysis completion
   */
  handleCodeAnalysisComplete(data) {
    this.logger.info('Code analysis completed', { 
      analysisId: data.analysisId,
      findings: data.findings?.length 
    });
    this.emit('codeAnalysisCompleted', data);
  }

  /**
   * Handle command execution
   */
  handleCommandExecuted(data) {
    this.logger.info('Command executed', { 
      command: data.command,
      exitCode: data.exitCode 
    });
    this.emit('commandExecutionTracked', data);
  }

  /**
   * Get agent by ID
   */
  getAgent(agentId) {
    return this.agents.get(agentId);
  }

  /**
   * Get all registered agents
   */
  getAgents() {
    return Array.from(this.agents.values());
  }

  /**
   * Get session by ID
   */
  getSession(sessionId) {
    return this.sessions.get(sessionId);
  }

  /**
   * Get all active sessions
   */
  getSessions() {
    return Array.from(this.sessions.values());
  }

  /**
   * Close session
   */
  async closeSession(sessionId) {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.status = 'closed';
      session.closedAt = new Date();
      
      const agent = this.agents.get(session.agentId);
      if (agent) {
        agent.sessions.delete(sessionId);
      }

      this.sessions.delete(sessionId);
      this.logger.info('Session closed', { sessionId });
      this.emit('sessionClosed', session);
    }
  }

  /**
   * Disconnect agent
   */
  async disconnectAgent(agentId) {
    const agent = this.agents.get(agentId);
    if (agent && agent.mcpConnection) {
      await this.mcpService.closeConnection(agent.mcpConnection.id);
      agent.status = 'disconnected';
      agent.mcpConnection = null;
      
      this.logger.info('Agent disconnected', { agentId });
      this.emit('agentDisconnected', agent);
    }
  }

  /**
   * Get service health status
   */
  getHealthStatus() {
    const agents = this.getAgents();
    const sessions = this.getSessions();
    
    return {
      status: 'healthy',
      agents: {
        total: agents.length,
        connected: agents.filter(a => a.status === 'connected').length,
        registered: agents.filter(a => a.status === 'registered').length,
        error: agents.filter(a => a.status === 'error').length
      },
      sessions: {
        total: sessions.length,
        active: sessions.filter(s => s.status === 'active').length
      },
      webSocketConnections: this.webSocketConnections.size,
      uptime: process.uptime()
    };
  }

  /**
   * Get service metrics
   */
  getMetrics() {
    const agents = this.getAgents();
    const totalRequests = agents.reduce((sum, agent) => sum + agent.metrics.totalRequests, 0);
    const totalWebhooks = agents.reduce((sum, agent) => sum + agent.metrics.totalWebhooks, 0);
    const totalErrors = agents.reduce((sum, agent) => sum + agent.metrics.errors, 0);

    return {
      agents: agents.length,
      sessions: this.sessions.size,
      totalRequests,
      totalWebhooks,
      totalErrors,
      uptime: process.uptime(),
      agentMetrics: agents.map(agent => ({
        id: agent.id,
        status: agent.status,
        sessions: agent.sessions.size,
        metrics: agent.metrics
      }))
    };
  }
}
