/**
 * AI Agent Rules and Policies
 * Defines rules, policies, and constraints for AI agent interactions
 */

// Agent Connection Rules
export const ConnectionRules = {
  // Maximum concurrent connections per agent type
  MAX_CONNECTIONS: {
    'github-copilot': 10,
    'amazon-q': 5,
    'cline': 3,
    'generic': 2
  },

  // Connection timeout settings
  TIMEOUTS: {
    CONNECTION: 30000,      // 30 seconds
    HANDSHAKE: 10000,       // 10 seconds
    HEARTBEAT: 60000,       // 1 minute
    IDLE: 300000,           // 5 minutes
    MAX_SESSION: 3600000    // 1 hour
  },

  // Rate limiting rules
  RATE_LIMITS: {
    REQUESTS_PER_MINUTE: 60,
    REQUESTS_PER_HOUR: 1000,
    WEBHOOK_CALLS_PER_MINUTE: 30,
    TOOL_CALLS_PER_MINUTE: 20
  },

  // Authentication requirements
  AUTHENTICATION: {
    REQUIRED: true,
    TOKEN_EXPIRY: 86400000, // 24 hours
    REFRESH_THRESHOLD: 3600000, // 1 hour before expiry
    MAX_FAILED_ATTEMPTS: 3
  }
};

// Data Access Rules
export const DataAccessRules = {
  // Resource access permissions
  RESOURCE_PERMISSIONS: {
    'github-copilot': {
      allowedResources: ['code', 'documentation', 'repositories'],
      deniedResources: ['secrets', 'credentials', 'private-keys'],
      maxResourceSize: 10485760, // 10MB
      allowedFileTypes: ['.js', '.ts', '.py', '.rs', '.go', '.java', '.cpp', '.c', '.md', '.txt']
    },
    'amazon-q': {
      allowedResources: ['code', 'documentation', 'logs', 'metrics'],
      deniedResources: ['secrets', 'credentials', 'private-keys', 'user-data'],
      maxResourceSize: 5242880, // 5MB
      allowedFileTypes: ['.js', '.ts', '.py', '.rs', '.go', '.java', '.cpp', '.c', '.md', '.txt', '.log']
    },
    'cline': {
      allowedResources: ['files', 'directories', 'terminal'],
      deniedResources: ['system-files', 'secrets', 'credentials'],
      maxResourceSize: 1048576, // 1MB
      allowedFileTypes: ['.js', '.ts', '.py', '.rs', '.sh', '.bat', '.md', '.txt', '.json', '.yaml']
    }
  },

  // Context sharing rules
  CONTEXT_SHARING: {
    CROSS_AGENT: false,        // Don't share context between different agents
    CROSS_SESSION: false,      // Don't share context between sessions
    PERSISTENT: true,          // Allow persistent context within session
    MAX_CONTEXT_SIZE: 1048576, // 1MB max context size
    RETENTION_PERIOD: 86400000 // 24 hours
  },

  // Data retention policies
  DATA_RETENTION: {
    CONVERSATION_HISTORY: 2592000000, // 30 days
    TOOL_CALL_LOGS: 604800000,        // 7 days
    ERROR_LOGS: 2592000000,           // 30 days
    METRICS: 7776000000,              // 90 days
    WEBHOOK_LOGS: 604800000           // 7 days
  }
};

// Security Rules
export const SecurityRules = {
  // Input validation rules
  INPUT_VALIDATION: {
    MAX_MESSAGE_LENGTH: 32768,     // 32KB
    MAX_TOOL_ARGS_SIZE: 8192,      // 8KB
    ALLOWED_PROTOCOLS: ['https', 'wss'],
    BLOCKED_PATTERNS: [
      /password/i,
      /secret/i,
      /token/i,
      /key/i,
      /credential/i,
      /api[_-]?key/i
    ]
  },

  // Output sanitization rules
  OUTPUT_SANITIZATION: {
    REMOVE_SECRETS: true,
    MASK_CREDENTIALS: true,
    FILTER_SYSTEM_INFO: true,
    MAX_OUTPUT_SIZE: 65536 // 64KB
  },

  // Webhook security
  WEBHOOK_SECURITY: {
    REQUIRE_SIGNATURE: true,
    SIGNATURE_ALGORITHMS: ['sha256', 'sha1'],
    MAX_PAYLOAD_SIZE: 1048576, // 1MB
    ALLOWED_IPS: [], // Empty means allow all, populate for IP restrictions
    RATE_LIMIT_PER_IP: 100 // requests per hour
  },

  // Tool execution security
  TOOL_EXECUTION: {
    SANDBOX_ENABLED: true,
    TIMEOUT: 30000, // 30 seconds
    MAX_MEMORY: 134217728, // 128MB
    ALLOWED_COMMANDS: [], // Whitelist of allowed commands
    BLOCKED_COMMANDS: ['rm', 'del', 'format', 'sudo', 'su']
  }
};

// Quality Rules
export const QualityRules = {
  // Response quality requirements
  RESPONSE_QUALITY: {
    MIN_CONFIDENCE: 0.7,
    MAX_RESPONSE_TIME: 5000, // 5 seconds
    REQUIRE_CITATIONS: true,
    MAX_HALLUCINATION_SCORE: 0.3
  },

  // Code quality requirements
  CODE_QUALITY: {
    REQUIRE_SYNTAX_CHECK: true,
    REQUIRE_LINTING: true,
    MAX_COMPLEXITY: 10,
    REQUIRE_TESTS: false,
    REQUIRE_DOCUMENTATION: true
  },

  // Content filtering
  CONTENT_FILTERING: {
    BLOCK_HARMFUL_CONTENT: true,
    BLOCK_PERSONAL_INFO: true,
    BLOCK_COPYRIGHTED_CODE: true,
    REQUIRE_ATTRIBUTION: true
  }
};

// Monitoring Rules
export const MonitoringRules = {
  // Performance monitoring
  PERFORMANCE: {
    TRACK_RESPONSE_TIMES: true,
    TRACK_ERROR_RATES: true,
    TRACK_RESOURCE_USAGE: true,
    ALERT_THRESHOLDS: {
      RESPONSE_TIME: 10000, // 10 seconds
      ERROR_RATE: 0.05,     // 5%
      CPU_USAGE: 0.8,       // 80%
      MEMORY_USAGE: 0.9     // 90%
    }
  },

  // Usage monitoring
  USAGE: {
    TRACK_API_CALLS: true,
    TRACK_TOOL_USAGE: true,
    TRACK_RESOURCE_ACCESS: true,
    DAILY_LIMITS: {
      API_CALLS: 10000,
      TOOL_CALLS: 1000,
      RESOURCE_READS: 5000
    }
  },

  // Audit logging
  AUDIT: {
    LOG_ALL_REQUESTS: true,
    LOG_TOOL_EXECUTIONS: true,
    LOG_RESOURCE_ACCESS: true,
    LOG_AUTHENTICATION: true,
    RETENTION_PERIOD: 7776000000 // 90 days
  }
};

// Compliance Rules
export const ComplianceRules = {
  // Privacy compliance
  PRIVACY: {
    GDPR_COMPLIANT: true,
    CCPA_COMPLIANT: true,
    DATA_MINIMIZATION: true,
    RIGHT_TO_DELETE: true,
    CONSENT_REQUIRED: true
  },

  // Security compliance
  SECURITY: {
    SOC2_COMPLIANT: true,
    ISO27001_COMPLIANT: true,
    ENCRYPTION_AT_REST: true,
    ENCRYPTION_IN_TRANSIT: true,
    ACCESS_LOGGING: true
  },

  // Industry compliance
  INDUSTRY: {
    HIPAA_COMPLIANT: false, // Set to true if handling health data
    PCI_COMPLIANT: false,   // Set to true if handling payment data
    SOX_COMPLIANT: false    // Set to true if handling financial data
  }
};

/**
 * Rule Validator
 * Validates agent actions against defined rules
 */
export class RuleValidator {
  constructor(logger) {
    this.logger = logger;
  }

  // Validate connection request
  validateConnection(agentType, connectionCount) {
    const maxConnections = ConnectionRules.MAX_CONNECTIONS[agentType] || ConnectionRules.MAX_CONNECTIONS.generic;
    
    if (connectionCount >= maxConnections) {
      return {
        valid: false,
        reason: `Maximum connections exceeded for ${agentType}`,
        limit: maxConnections
      };
    }

    return { valid: true };
  }

  // Validate resource access
  validateResourceAccess(agentType, resourceType, resourceSize, fileType) {
    const permissions = DataAccessRules.RESOURCE_PERMISSIONS[agentType];
    
    if (!permissions) {
      return { valid: false, reason: 'Unknown agent type' };
    }

    // Check allowed resources
    if (!permissions.allowedResources.includes(resourceType)) {
      return { valid: false, reason: `Resource type '${resourceType}' not allowed` };
    }

    // Check denied resources
    if (permissions.deniedResources.includes(resourceType)) {
      return { valid: false, reason: `Resource type '${resourceType}' explicitly denied` };
    }

    // Check resource size
    if (resourceSize > permissions.maxResourceSize) {
      return { valid: false, reason: 'Resource size exceeds limit' };
    }

    // Check file type
    if (fileType && !permissions.allowedFileTypes.includes(fileType)) {
      return { valid: false, reason: `File type '${fileType}' not allowed` };
    }

    return { valid: true };
  }

  // Validate input content
  validateInput(content) {
    // Check length
    if (content.length > SecurityRules.INPUT_VALIDATION.MAX_MESSAGE_LENGTH) {
      return { valid: false, reason: 'Input too long' };
    }

    // Check for blocked patterns
    for (const pattern of SecurityRules.INPUT_VALIDATION.BLOCKED_PATTERNS) {
      if (pattern.test(content)) {
        return { valid: false, reason: 'Input contains blocked content' };
      }
    }

    return { valid: true };
  }

  // Validate tool execution
  validateToolExecution(toolName, args, agentType) {
    const permissions = DataAccessRules.RESOURCE_PERMISSIONS[agentType];
    
    if (!permissions) {
      return { valid: false, reason: 'Unknown agent type' };
    }

    // Check if tool execution is allowed for this agent
    if (SecurityRules.TOOL_EXECUTION.BLOCKED_COMMANDS.includes(toolName)) {
      return { valid: false, reason: `Tool '${toolName}' is blocked` };
    }

    // Check argument size
    const argsSize = JSON.stringify(args).length;
    if (argsSize > SecurityRules.INPUT_VALIDATION.MAX_TOOL_ARGS_SIZE) {
      return { valid: false, reason: 'Tool arguments too large' };
    }

    return { valid: true };
  }

  // Validate webhook request
  validateWebhook(payload, signature, agentType) {
    // Check payload size
    if (payload.length > SecurityRules.WEBHOOK_SECURITY.MAX_PAYLOAD_SIZE) {
      return { valid: false, reason: 'Webhook payload too large' };
    }

    // Check signature if required
    if (SecurityRules.WEBHOOK_SECURITY.REQUIRE_SIGNATURE && !signature) {
      return { valid: false, reason: 'Webhook signature required' };
    }

    return { valid: true };
  }

  // Check rate limits
  checkRateLimit(agentType, requestType, currentCount, timeWindow) {
    const limits = ConnectionRules.RATE_LIMITS;
    let limit;

    switch (requestType) {
      case 'request':
        limit = timeWindow === 'minute' ? limits.REQUESTS_PER_MINUTE : limits.REQUESTS_PER_HOUR;
        break;
      case 'webhook':
        limit = limits.WEBHOOK_CALLS_PER_MINUTE;
        break;
      case 'tool':
        limit = limits.TOOL_CALLS_PER_MINUTE;
        break;
      default:
        return { valid: false, reason: 'Unknown request type' };
    }

    if (currentCount >= limit) {
      return {
        valid: false,
        reason: `Rate limit exceeded for ${requestType}`,
        limit,
        resetTime: Date.now() + (timeWindow === 'minute' ? 60000 : 3600000)
      };
    }

    return { valid: true, remaining: limit - currentCount };
  }
}

/**
 * Rule Engine
 * Main engine for applying and enforcing rules
 */
export class RuleEngine {
  constructor(logger) {
    this.logger = logger;
    this.validator = new RuleValidator(logger);
    this.rateLimiters = new Map();
  }

  // Apply all relevant rules to an agent action
  async applyRules(action, context) {
    const results = [];

    switch (action.type) {
      case 'connect':
        results.push(this.validator.validateConnection(action.agentType, action.connectionCount));
        break;

      case 'resource_access':
        results.push(this.validator.validateResourceAccess(
          action.agentType,
          action.resourceType,
          action.resourceSize,
          action.fileType
        ));
        break;

      case 'tool_execution':
        results.push(this.validator.validateToolExecution(
          action.toolName,
          action.args,
          action.agentType
        ));
        break;

      case 'webhook':
        results.push(this.validator.validateWebhook(
          action.payload,
          action.signature,
          action.agentType
        ));
        break;

      case 'input':
        results.push(this.validator.validateInput(action.content));
        break;
    }

    // Check rate limits
    const rateLimitResult = this.checkAndUpdateRateLimit(
      action.agentType,
      action.type,
      context.timeWindow || 'minute'
    );
    results.push(rateLimitResult);

    // Return combined results
    const failedRules = results.filter(r => !r.valid);
    
    if (failedRules.length > 0) {
      return {
        allowed: false,
        violations: failedRules,
        message: failedRules.map(r => r.reason).join('; ')
      };
    }

    return {
      allowed: true,
      violations: [],
      message: 'All rules passed'
    };
  }

  // Check and update rate limits
  checkAndUpdateRateLimit(agentType, requestType, timeWindow) {
    const key = `${agentType}:${requestType}:${timeWindow}`;
    const now = Date.now();
    const windowMs = timeWindow === 'minute' ? 60000 : 3600000;
    
    if (!this.rateLimiters.has(key)) {
      this.rateLimiters.set(key, { count: 0, resetTime: now + windowMs });
    }

    const limiter = this.rateLimiters.get(key);
    
    // Reset if window expired
    if (now >= limiter.resetTime) {
      limiter.count = 0;
      limiter.resetTime = now + windowMs;
    }

    // Check limit
    const result = this.validator.checkRateLimit(agentType, requestType, limiter.count, timeWindow);
    
    if (result.valid) {
      limiter.count++;
    }

    return result;
  }

  // Get current rule configuration
  getRuleConfiguration() {
    return {
      connection: ConnectionRules,
      dataAccess: DataAccessRules,
      security: SecurityRules,
      quality: QualityRules,
      monitoring: MonitoringRules,
      compliance: ComplianceRules
    };
  }

  // Update rule configuration
  updateRuleConfiguration(ruleType, updates) {
    switch (ruleType) {
      case 'connection':
        Object.assign(ConnectionRules, updates);
        break;
      case 'dataAccess':
        Object.assign(DataAccessRules, updates);
        break;
      case 'security':
        Object.assign(SecurityRules, updates);
        break;
      case 'quality':
        Object.assign(QualityRules, updates);
        break;
      case 'monitoring':
        Object.assign(MonitoringRules, updates);
        break;
      case 'compliance':
        Object.assign(ComplianceRules, updates);
        break;
      default:
        throw new Error(`Unknown rule type: ${ruleType}`);
    }

    this.logger.info(`Updated ${ruleType} rules`, updates);
  }
}
