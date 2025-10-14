


export const ConnectionRules = {
  
  MAX_CONNECTIONS: {
    'github-copilot': 10,
    'amazon-q': 5,
    'cline': 3,
    'generic': 2
  },

  
  TIMEOUTS: {
    CONNECTION: 30000,      
    HANDSHAKE: 10000,       
    HEARTBEAT: 60000,       
    IDLE: 300000,           
    MAX_SESSION: 3600000    
  },

  
  RATE_LIMITS: {
    REQUESTS_PER_MINUTE: 60,
    REQUESTS_PER_HOUR: 1000,
    WEBHOOK_CALLS_PER_MINUTE: 30,
    TOOL_CALLS_PER_MINUTE: 20
  },

  
  AUTHENTICATION: {
    REQUIRED: true,
    TOKEN_EXPIRY: 86400000, 
    REFRESH_THRESHOLD: 3600000, 
    MAX_FAILED_ATTEMPTS: 3
  }
};


export const DataAccessRules = {
  
  RESOURCE_PERMISSIONS: {
    'github-copilot': {
      allowedResources: ['code', 'documentation', 'repositories'],
      deniedResources: ['secrets', 'credentials', 'private-keys'],
      maxResourceSize: 10485760, 
      allowedFileTypes: ['.js', '.ts', '.py', '.rs', '.go', '.java', '.cpp', '.c', '.md', '.txt']
    },
    'amazon-q': {
      allowedResources: ['code', 'documentation', 'logs', 'metrics'],
      deniedResources: ['secrets', 'credentials', 'private-keys', 'user-data'],
      maxResourceSize: 5242880, 
      allowedFileTypes: ['.js', '.ts', '.py', '.rs', '.go', '.java', '.cpp', '.c', '.md', '.txt', '.log']
    },
    'cline': {
      allowedResources: ['files', 'directories', 'terminal'],
      deniedResources: ['system-files', 'secrets', 'credentials'],
      maxResourceSize: 1048576, 
      allowedFileTypes: ['.js', '.ts', '.py', '.rs', '.sh', '.bat', '.md', '.txt', '.json', '.yaml']
    }
  },

  
  CONTEXT_SHARING: {
    CROSS_AGENT: false,        
    CROSS_SESSION: false,      
    PERSISTENT: true,          
    MAX_CONTEXT_SIZE: 1048576, 
    RETENTION_PERIOD: 86400000 
  },

  
  DATA_RETENTION: {
    CONVERSATION_HISTORY: 2592000000, 
    TOOL_CALL_LOGS: 604800000,        
    ERROR_LOGS: 2592000000,           
    METRICS: 7776000000,              
    WEBHOOK_LOGS: 604800000           
  }
};


export const SecurityRules = {
  
  INPUT_VALIDATION: {
    MAX_MESSAGE_LENGTH: 32768,     
    MAX_TOOL_ARGS_SIZE: 8192,      
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

  
  OUTPUT_SANITIZATION: {
    REMOVE_SECRETS: true,
    MASK_CREDENTIALS: true,
    FILTER_SYSTEM_INFO: true,
    MAX_OUTPUT_SIZE: 65536 
  },

  
  WEBHOOK_SECURITY: {
    REQUIRE_SIGNATURE: true,
    SIGNATURE_ALGORITHMS: ['sha256', 'sha1'],
    MAX_PAYLOAD_SIZE: 1048576, 
    ALLOWED_IPS: [], 
    RATE_LIMIT_PER_IP: 100 
  },

  
  TOOL_EXECUTION: {
    SANDBOX_ENABLED: true,
    TIMEOUT: 30000, 
    MAX_MEMORY: 134217728, 
    ALLOWED_COMMANDS: [], 
    BLOCKED_COMMANDS: ['rm', 'del', 'format', 'sudo', 'su']
  }
};


export const QualityRules = {
  
  RESPONSE_QUALITY: {
    MIN_CONFIDENCE: 0.7,
    MAX_RESPONSE_TIME: 5000, 
    REQUIRE_CITATIONS: true,
    MAX_HALLUCINATION_SCORE: 0.3
  },

  
  CODE_QUALITY: {
    REQUIRE_SYNTAX_CHECK: true,
    REQUIRE_LINTING: true,
    MAX_COMPLEXITY: 10,
    REQUIRE_TESTS: false,
    REQUIRE_DOCUMENTATION: true
  },

  
  CONTENT_FILTERING: {
    BLOCK_HARMFUL_CONTENT: true,
    BLOCK_PERSONAL_INFO: true,
    BLOCK_COPYRIGHTED_CODE: true,
    REQUIRE_ATTRIBUTION: true
  }
};


export const MonitoringRules = {
  
  PERFORMANCE: {
    TRACK_RESPONSE_TIMES: true,
    TRACK_ERROR_RATES: true,
    TRACK_RESOURCE_USAGE: true,
    ALERT_THRESHOLDS: {
      RESPONSE_TIME: 10000, 
      ERROR_RATE: 0.05,     
      CPU_USAGE: 0.8,       
      MEMORY_USAGE: 0.9     
    }
  },

  
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

  
  AUDIT: {
    LOG_ALL_REQUESTS: true,
    LOG_TOOL_EXECUTIONS: true,
    LOG_RESOURCE_ACCESS: true,
    LOG_AUTHENTICATION: true,
    RETENTION_PERIOD: 7776000000 
  }
};


export const ComplianceRules = {
  
  PRIVACY: {
    GDPR_COMPLIANT: true,
    CCPA_COMPLIANT: true,
    DATA_MINIMIZATION: true,
    RIGHT_TO_DELETE: true,
    CONSENT_REQUIRED: true
  },

  
  SECURITY: {
    SOC2_COMPLIANT: true,
    ISO27001_COMPLIANT: true,
    ENCRYPTION_AT_REST: true,
    ENCRYPTION_IN_TRANSIT: true,
    ACCESS_LOGGING: true
  },

  
  INDUSTRY: {
    HIPAA_COMPLIANT: false, 
    PCI_COMPLIANT: false,   
    SOX_COMPLIANT: false    
  }
};


export class RuleValidator {
  constructor(logger) {
    this.logger = logger;
  }

  
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

  
  validateResourceAccess(agentType, resourceType, resourceSize, fileType) {
    const permissions = DataAccessRules.RESOURCE_PERMISSIONS[agentType];
    
    if (!permissions) {
      return { valid: false, reason: 'Unknown agent type' };
    }

    
    if (!permissions.allowedResources.includes(resourceType)) {
      return { valid: false, reason: `Resource type '${resourceType}' not allowed` };
    }

    
    if (permissions.deniedResources.includes(resourceType)) {
      return { valid: false, reason: `Resource type '${resourceType}' explicitly denied` };
    }

    
    if (resourceSize > permissions.maxResourceSize) {
      return { valid: false, reason: 'Resource size exceeds limit' };
    }

    
    if (fileType && !permissions.allowedFileTypes.includes(fileType)) {
      return { valid: false, reason: `File type '${fileType}' not allowed` };
    }

    return { valid: true };
  }

  
  validateInput(content) {
    
    if (content.length > SecurityRules.INPUT_VALIDATION.MAX_MESSAGE_LENGTH) {
      return { valid: false, reason: 'Input too long' };
    }

    
    for (const pattern of SecurityRules.INPUT_VALIDATION.BLOCKED_PATTERNS) {
      if (pattern.test(content)) {
        return { valid: false, reason: 'Input contains blocked content' };
      }
    }

    return { valid: true };
  }

  
  validateToolExecution(toolName, args, agentType) {
    const permissions = DataAccessRules.RESOURCE_PERMISSIONS[agentType];
    
    if (!permissions) {
      return { valid: false, reason: 'Unknown agent type' };
    }

    
    if (SecurityRules.TOOL_EXECUTION.BLOCKED_COMMANDS.includes(toolName)) {
      return { valid: false, reason: `Tool '${toolName}' is blocked` };
    }

    
    const argsSize = JSON.stringify(args).length;
    if (argsSize > SecurityRules.INPUT_VALIDATION.MAX_TOOL_ARGS_SIZE) {
      return { valid: false, reason: 'Tool arguments too large' };
    }

    return { valid: true };
  }

  
  validateWebhook(payload, signature, agentType) {
    
    if (payload.length > SecurityRules.WEBHOOK_SECURITY.MAX_PAYLOAD_SIZE) {
      return { valid: false, reason: 'Webhook payload too large' };
    }

    
    if (SecurityRules.WEBHOOK_SECURITY.REQUIRE_SIGNATURE && !signature) {
      return { valid: false, reason: 'Webhook signature required' };
    }

    return { valid: true };
  }

  
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


export class RuleEngine {
  constructor(logger) {
    this.logger = logger;
    this.validator = new RuleValidator(logger);
    this.rateLimiters = new Map();
  }

  
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

    
    const rateLimitResult = this.checkAndUpdateRateLimit(
      action.agentType,
      action.type,
      context.timeWindow || 'minute'
    );
    results.push(rateLimitResult);

    
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

  
  checkAndUpdateRateLimit(agentType, requestType, timeWindow) {
    const key = `${agentType}:${requestType}:${timeWindow}`;
    const now = Date.now();
    const windowMs = timeWindow === 'minute' ? 60000 : 3600000;
    
    if (!this.rateLimiters.has(key)) {
      this.rateLimiters.set(key, { count: 0, resetTime: now + windowMs });
    }

    const limiter = this.rateLimiters.get(key);
    
    
    if (now >= limiter.resetTime) {
      limiter.count = 0;
      limiter.resetTime = now + windowMs;
    }

    
    const result = this.validator.checkRateLimit(agentType, requestType, limiter.count, timeWindow);
    
    if (result.valid) {
      limiter.count++;
    }

    return result;
  }

  
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
