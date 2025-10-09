/**
 * Model Context Protocol (MCP) Implementation
 * Core protocol definitions and message handling for AI agent communication
 */

export const MCP_VERSION = '2024-11-05';

// MCP Message Types
export const MessageType = {
  // Core protocol messages
  INITIALIZE: 'initialize',
  INITIALIZED: 'initialized',
  
  // Resource management
  RESOURCES_LIST: 'resources/list',
  RESOURCES_READ: 'resources/read',
  RESOURCES_SUBSCRIBE: 'resources/subscribe',
  RESOURCES_UNSUBSCRIBE: 'resources/unsubscribe',
  RESOURCES_UPDATED: 'resources/updated',
  
  // Tool management
  TOOLS_LIST: 'tools/list',
  TOOLS_CALL: 'tools/call',
  
  // Prompt management
  PROMPTS_LIST: 'prompts/list',
  PROMPTS_GET: 'prompts/get',
  
  // Logging
  LOGGING_SET_LEVEL: 'logging/setLevel',
  
  // Notifications
  NOTIFICATION: 'notification',
  
  // Progress reporting
  PROGRESS: 'progress',
  
  // Completion
  COMPLETION_COMPLETE: 'completion/complete'
};

// MCP Error Codes (JSON-RPC 2.0 compatible)
export const ErrorCode = {
  PARSE_ERROR: -32700,
  INVALID_REQUEST: -32600,
  METHOD_NOT_FOUND: -32601,
  INVALID_PARAMS: -32602,
  INTERNAL_ERROR: -32603,
  
  // MCP-specific errors
  RESOURCE_NOT_FOUND: -32001,
  TOOL_NOT_FOUND: -32002,
  PROMPT_NOT_FOUND: -32003,
  CAPABILITY_NOT_SUPPORTED: -32004,
  AUTHENTICATION_FAILED: -32005,
  RATE_LIMITED: -32006,
  TIMEOUT: -32007
};

// MCP Capability Types
export const CapabilityType = {
  RESOURCES: 'resources',
  TOOLS: 'tools',
  PROMPTS: 'prompts',
  LOGGING: 'logging',
  COMPLETION: 'completion'
};

// Resource Types
export const ResourceType = {
  FILE: 'file',
  DIRECTORY: 'directory',
  URL: 'url',
  DATABASE: 'database',
  API: 'api',
  MEMORY: 'memory',
  CONTEXT: 'context'
};

// Tool Categories
export const ToolCategory = {
  CODE_GENERATION: 'code_generation',
  CODE_ANALYSIS: 'code_analysis',
  DOCUMENTATION: 'documentation',
  TESTING: 'testing',
  DEBUGGING: 'debugging',
  REFACTORING: 'refactoring',
  SECURITY: 'security',
  PERFORMANCE: 'performance',
  DEPLOYMENT: 'deployment',
  MONITORING: 'monitoring'
};

/**
 * MCP Message Builder
 * Utility class for creating properly formatted MCP messages
 */
export class McpMessageBuilder {
  static createRequest(id, method, params = {}) {
    return {
      jsonrpc: '2.0',
      id,
      method,
      params
    };
  }

  static createResponse(id, result) {
    return {
      jsonrpc: '2.0',
      id,
      result
    };
  }

  static createError(id, code, message, data = null) {
    const error = {
      jsonrpc: '2.0',
      id,
      error: {
        code,
        message
      }
    };
    
    if (data) {
      error.error.data = data;
    }
    
    return error;
  }

  static createNotification(method, params = {}) {
    return {
      jsonrpc: '2.0',
      method,
      params
    };
  }

  static createInitializeRequest(clientInfo, capabilities) {
    return this.createRequest('init', MessageType.INITIALIZE, {
      protocolVersion: MCP_VERSION,
      capabilities,
      clientInfo
    });
  }

  static createResourceListRequest() {
    return this.createRequest('list-resources', MessageType.RESOURCES_LIST);
  }

  static createResourceReadRequest(uri) {
    return this.createRequest('read-resource', MessageType.RESOURCES_READ, { uri });
  }

  static createToolListRequest() {
    return this.createRequest('list-tools', MessageType.TOOLS_LIST);
  }

  static createToolCallRequest(name, arguments_) {
    return this.createRequest('call-tool', MessageType.TOOLS_CALL, {
      name,
      arguments: arguments_
    });
  }
}

/**
 * MCP Message Validator
 * Validates MCP messages according to the protocol specification
 */
export class McpMessageValidator {
  static validateMessage(message) {
    if (!message || typeof message !== 'object') {
      return { valid: false, error: 'Message must be an object' };
    }

    if (message.jsonrpc !== '2.0') {
      return { valid: false, error: 'Invalid JSON-RPC version' };
    }

    // Check if it's a request, response, or notification
    if (message.method) {
      return this.validateRequest(message);
    } else if (message.result !== undefined || message.error !== undefined) {
      return this.validateResponse(message);
    } else {
      return { valid: false, error: 'Invalid message format' };
    }
  }

  static validateRequest(message) {
    if (!message.method || typeof message.method !== 'string') {
      return { valid: false, error: 'Request must have a method' };
    }

    // Notification (no id required)
    if (message.id === undefined) {
      return { valid: true };
    }

    // Request (id required)
    if (message.id === null || message.id === undefined) {
      return { valid: false, error: 'Request must have an id' };
    }

    return { valid: true };
  }

  static validateResponse(message) {
    if (message.id === undefined) {
      return { valid: false, error: 'Response must have an id' };
    }

    if (message.result !== undefined && message.error !== undefined) {
      return { valid: false, error: 'Response cannot have both result and error' };
    }

    if (message.result === undefined && message.error === undefined) {
      return { valid: false, error: 'Response must have either result or error' };
    }

    if (message.error) {
      return this.validateError(message.error);
    }

    return { valid: true };
  }

  static validateError(error) {
    if (!error.code || typeof error.code !== 'number') {
      return { valid: false, error: 'Error must have a numeric code' };
    }

    if (!error.message || typeof error.message !== 'string') {
      return { valid: false, error: 'Error must have a message' };
    }

    return { valid: true };
  }
}

/**
 * MCP Capability Manager
 * Manages and negotiates capabilities between client and server
 */
export class McpCapabilityManager {
  constructor() {
    this.clientCapabilities = null;
    this.serverCapabilities = null;
    this.negotiatedCapabilities = null;
  }

  setClientCapabilities(capabilities) {
    this.clientCapabilities = capabilities;
    this.negotiateCapabilities();
  }

  setServerCapabilities(capabilities) {
    this.serverCapabilities = capabilities;
    this.negotiateCapabilities();
  }

  negotiateCapabilities() {
    if (!this.clientCapabilities || !this.serverCapabilities) {
      return;
    }

    this.negotiatedCapabilities = {};

    // Negotiate each capability type
    for (const [capType, clientCap] of Object.entries(this.clientCapabilities)) {
      const serverCap = this.serverCapabilities[capType];
      
      if (serverCap) {
        this.negotiatedCapabilities[capType] = this.negotiateCapability(clientCap, serverCap);
      }
    }
  }

  negotiateCapability(clientCap, serverCap) {
    if (typeof clientCap === 'boolean' && typeof serverCap === 'boolean') {
      return clientCap && serverCap;
    }

    if (typeof clientCap === 'object' && typeof serverCap === 'object') {
      const negotiated = {};
      
      for (const [feature, clientSupport] of Object.entries(clientCap)) {
        const serverSupport = serverCap[feature];
        
        if (serverSupport !== undefined) {
          if (typeof clientSupport === 'boolean' && typeof serverSupport === 'boolean') {
            negotiated[feature] = clientSupport && serverSupport;
          } else {
            negotiated[feature] = serverSupport;
          }
        }
      }
      
      return negotiated;
    }

    return serverCap;
  }

  hasCapability(capabilityType, feature = null) {
    if (!this.negotiatedCapabilities) {
      return false;
    }

    const capability = this.negotiatedCapabilities[capabilityType];
    
    if (!capability) {
      return false;
    }

    if (feature === null) {
      return !!capability;
    }

    if (typeof capability === 'boolean') {
      return capability;
    }

    if (typeof capability === 'object') {
      return !!capability[feature];
    }

    return false;
  }

  getCapabilities() {
    return this.negotiatedCapabilities;
  }
}

/**
 * Default MCP Capabilities
 */
export const DefaultCapabilities = {
  client: {
    resources: {
      subscribe: true,
      listChanged: true
    },
    tools: {
      listChanged: true
    },
    prompts: {
      listChanged: true
    },
    logging: {}
  },
  
  server: {
    resources: {
      subscribe: true,
      listChanged: true
    },
    tools: {
      listChanged: true
    },
    prompts: {
      listChanged: true
    },
    logging: {
      level: 'info'
    }
  }
};

/**
 * MCP Protocol Handler
 * Main class for handling MCP protocol communication
 */
export class McpProtocolHandler {
  constructor(logger) {
    this.logger = logger;
    this.capabilityManager = new McpCapabilityManager();
    this.messageHandlers = new Map();
    this.initialized = false;
    
    this.setupDefaultHandlers();
  }

  setupDefaultHandlers() {
    this.messageHandlers.set(MessageType.INITIALIZE, this.handleInitialize.bind(this));
    this.messageHandlers.set(MessageType.RESOURCES_LIST, this.handleResourcesList.bind(this));
    this.messageHandlers.set(MessageType.RESOURCES_READ, this.handleResourcesRead.bind(this));
    this.messageHandlers.set(MessageType.TOOLS_LIST, this.handleToolsList.bind(this));
    this.messageHandlers.set(MessageType.TOOLS_CALL, this.handleToolsCall.bind(this));
  }

  registerHandler(method, handler) {
    this.messageHandlers.set(method, handler);
  }

  async handleMessage(message) {
    const validation = McpMessageValidator.validateMessage(message);
    
    if (!validation.valid) {
      return McpMessageBuilder.createError(
        message.id || null,
        ErrorCode.INVALID_REQUEST,
        validation.error
      );
    }

    const handler = this.messageHandlers.get(message.method);
    
    if (!handler) {
      return McpMessageBuilder.createError(
        message.id || null,
        ErrorCode.METHOD_NOT_FOUND,
        `Method '${message.method}' not found`
      );
    }

    try {
      const result = await handler(message);
      
      if (message.id !== undefined) {
        return McpMessageBuilder.createResponse(message.id, result);
      }
      
      return null; // Notification, no response needed
    } catch (error) {
      this.logger.error('Error handling MCP message:', error);
      
      return McpMessageBuilder.createError(
        message.id || null,
        ErrorCode.INTERNAL_ERROR,
        error.message
      );
    }
  }

  async handleInitialize(message) {
    const { protocolVersion, capabilities, clientInfo } = message.params;
    
    if (protocolVersion !== MCP_VERSION) {
      throw new Error(`Unsupported protocol version: ${protocolVersion}`);
    }

    this.capabilityManager.setClientCapabilities(capabilities);
    this.capabilityManager.setServerCapabilities(DefaultCapabilities.server);
    
    this.initialized = true;
    
    this.logger.info('MCP connection initialized', { clientInfo });
    
    return {
      protocolVersion: MCP_VERSION,
      capabilities: this.capabilityManager.getCapabilities(),
      serverInfo: {
        name: 'ConHub MCP Server',
        version: '1.0.0'
      }
    };
  }

  async handleResourcesList(message) {
    // Override in subclass or register custom handler
    return { resources: [] };
  }

  async handleResourcesRead(message) {
    // Override in subclass or register custom handler
    const { uri } = message.params;
    throw new Error(`Resource not found: ${uri}`);
  }

  async handleToolsList(message) {
    // Override in subclass or register custom handler
    return { tools: [] };
  }

  async handleToolsCall(message) {
    // Override in subclass or register custom handler
    const { name } = message.params;
    throw new Error(`Tool not found: ${name}`);
  }

  isInitialized() {
    return this.initialized;
  }

  getCapabilities() {
    return this.capabilityManager.getCapabilities();
  }
}
