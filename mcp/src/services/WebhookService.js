import crypto from 'crypto';
import { EventEmitter } from 'events';


export class WebhookService extends EventEmitter {
  constructor(logger) {
    super();
    this.logger = logger;
    this.webhookHandlers = new Map();
    this.webhookSecrets = new Map();
    
    
    this.initializeWebhookHandlers();
    
    this.logger.info('Webhook Service initialized');
  }

  
  initializeWebhookHandlers() {
    
    this.webhookHandlers.set('github-copilot', {
      handler: this.handleGitHubCopilotWebhook.bind(this),
      secret: process.env.GITHUB_COPILOT_WEBHOOK_SECRET,
      signatureHeader: 'x-hub-signature-256'
    });

    
    this.webhookHandlers.set('amazon-q', {
      handler: this.handleAmazonQWebhook.bind(this),
      secret: process.env.AMAZON_Q_WEBHOOK_SECRET,
      signatureHeader: 'x-amz-signature'
    });

    
    this.webhookHandlers.set('cline', {
      handler: this.handleClineWebhook.bind(this),
      secret: process.env.CLINE_WEBHOOK_SECRET,
      signatureHeader: 'x-cline-signature'
    });

    
    this.webhookHandlers.set('generic', {
      handler: this.handleGenericWebhook.bind(this),
      secret: process.env.GENERIC_WEBHOOK_SECRET,
      signatureHeader: 'x-webhook-signature'
    });
  }

  
  verifyWebhookSignature(payload, signature, secret, algorithm = 'sha256') {
    if (!secret || !signature) {
      return false;
    }

    try {
      const expectedSignature = crypto
        .createHmac(algorithm, secret)
        .update(payload, 'utf8')
        .digest('hex');

      
      const cleanSignature = signature.replace(/^(sha256=|sha1=)/, '');
      
      return crypto.timingSafeEqual(
        Buffer.from(expectedSignature, 'hex'),
        Buffer.from(cleanSignature, 'hex')
      );
    } catch (error) {
      this.logger.error('Webhook signature verification failed', { error: error.message });
      return false;
    }
  }

  
  async processWebhook(agentType, headers, body, rawBody) {
    try {
      const handler = this.webhookHandlers.get(agentType);
      if (!handler) {
        throw new Error(`No webhook handler found for agent type: ${agentType}`);
      }

      
      if (handler.secret && handler.signatureHeader) {
        const signature = headers[handler.signatureHeader];
        const isValid = this.verifyWebhookSignature(rawBody, signature, handler.secret);
        
        if (!isValid) {
          throw new Error('Invalid webhook signature');
        }
      }

      
      const result = await handler.handler(headers, body);
      
      this.logger.info('Webhook processed successfully', { 
        agentType, 
        eventType: body.event_type || body.action || 'unknown'
      });

      
      this.emit('webhookReceived', {
        agentType,
        headers,
        body,
        result,
        timestamp: new Date()
      });

      return result;
    } catch (error) {
      this.logger.error('Webhook processing failed', { 
        agentType, 
        error: error.message 
      });
      throw error;
    }
  }

  
  async handleGitHubCopilotWebhook(headers, body) {
    const eventType = headers['x-github-event'] || body.action;
    
    switch (eventType) {
      case 'copilot_usage':
        return this.handleCopilotUsageEvent(body);
      
      case 'copilot_suggestion':
        return this.handleCopilotSuggestionEvent(body);
      
      case 'copilot_chat':
        return this.handleCopilotChatEvent(body);
      
      case 'repository':
        return this.handleRepositoryEvent(body);
      
      case 'push':
        return this.handlePushEvent(body);
      
      default:
        this.logger.warn('Unknown GitHub Copilot event type', { eventType });
        return { status: 'ignored', eventType };
    }
  }

  
  async handleAmazonQWebhook(headers, body) {
    const eventType = headers['x-amz-event-type'] || body.eventType;
    
    switch (eventType) {
      case 'code_analysis_complete':
        return this.handleCodeAnalysisComplete(body);
      
      case 'security_scan_complete':
        return this.handleSecurityScanComplete(body);
      
      case 'recommendation_generated':
        return this.handleRecommendationGenerated(body);
      
      case 'chat_interaction':
        return this.handleQChatInteraction(body);
      
      default:
        this.logger.warn('Unknown Amazon Q event type', { eventType });
        return { status: 'ignored', eventType };
    }
  }

  
  async handleClineWebhook(headers, body) {
    const eventType = headers['x-cline-event'] || body.event;
    
    switch (eventType) {
      case 'command_executed':
        return this.handleCommandExecuted(body);
      
      case 'file_modified':
        return this.handleFileModified(body);
      
      case 'task_completed':
        return this.handleTaskCompleted(body);
      
      case 'error_occurred':
        return this.handleErrorOccurred(body);
      
      default:
        this.logger.warn('Unknown Cline event type', { eventType });
        return { status: 'ignored', eventType };
    }
  }

  
  async handleGenericWebhook(headers, body) {
    const eventType = headers['x-event-type'] || body.type || 'generic';
    
    this.logger.info('Processing generic webhook', { eventType, body });
    
    
    const webhookData = {
      id: crypto.randomUUID(),
      agentType: 'generic',
      eventType,
      headers,
      body,
      timestamp: new Date(),
      processed: false
    };

    
    this.emit('genericWebhook', webhookData);
    
    return { status: 'received', id: webhookData.id };
  }

  
  async handleCopilotUsageEvent(body) {
    this.logger.info('Copilot usage event received', { 
      user: body.user?.login,
      usage: body.usage 
    });
    
    
    this.emit('copilotUsage', {
      user: body.user,
      usage: body.usage,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'usage' };
  }

  async handleCopilotSuggestionEvent(body) {
    this.logger.info('Copilot suggestion event received', {
      repository: body.repository?.name,
      suggestions: body.suggestions?.length || 0
    });
    
    
    this.emit('copilotSuggestion', {
      repository: body.repository,
      suggestions: body.suggestions,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'suggestion' };
  }

  async handleCopilotChatEvent(body) {
    this.logger.info('Copilot chat event received', {
      user: body.user?.login,
      messageCount: body.messages?.length || 0
    });
    
    
    this.emit('copilotChat', {
      user: body.user,
      messages: body.messages,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'chat' };
  }

  async handleRepositoryEvent(body) {
    this.logger.info('Repository event received', {
      action: body.action,
      repository: body.repository?.name
    });
    
    
    this.emit('repositoryChange', {
      action: body.action,
      repository: body.repository,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'repository' };
  }

  async handlePushEvent(body) {
    this.logger.info('Push event received', {
      repository: body.repository?.name,
      commits: body.commits?.length || 0,
      ref: body.ref
    });
    
    
    this.emit('codePush', {
      repository: body.repository,
      commits: body.commits,
      ref: body.ref,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'push' };
  }

  
  async handleCodeAnalysisComplete(body) {
    this.logger.info('Amazon Q code analysis complete', {
      analysisId: body.analysisId,
      findings: body.findings?.length || 0
    });
    
    this.emit('codeAnalysisComplete', {
      analysisId: body.analysisId,
      findings: body.findings,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'code_analysis' };
  }

  async handleSecurityScanComplete(body) {
    this.logger.info('Amazon Q security scan complete', {
      scanId: body.scanId,
      vulnerabilities: body.vulnerabilities?.length || 0
    });
    
    this.emit('securityScanComplete', {
      scanId: body.scanId,
      vulnerabilities: body.vulnerabilities,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'security_scan' };
  }

  async handleRecommendationGenerated(body) {
    this.logger.info('Amazon Q recommendation generated', {
      recommendationId: body.recommendationId,
      type: body.type
    });
    
    this.emit('recommendationGenerated', {
      recommendationId: body.recommendationId,
      recommendation: body.recommendation,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'recommendation' };
  }

  async handleQChatInteraction(body) {
    this.logger.info('Amazon Q chat interaction', {
      sessionId: body.sessionId,
      messageId: body.messageId
    });
    
    this.emit('qChatInteraction', {
      sessionId: body.sessionId,
      message: body.message,
      response: body.response,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'chat_interaction' };
  }

  
  async handleCommandExecuted(body) {
    this.logger.info('Cline command executed', {
      command: body.command,
      exitCode: body.exitCode
    });
    
    this.emit('commandExecuted', {
      command: body.command,
      output: body.output,
      exitCode: body.exitCode,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'command_executed' };
  }

  async handleFileModified(body) {
    this.logger.info('Cline file modified', {
      filePath: body.filePath,
      operation: body.operation
    });
    
    this.emit('fileModified', {
      filePath: body.filePath,
      operation: body.operation,
      changes: body.changes,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'file_modified' };
  }

  async handleTaskCompleted(body) {
    this.logger.info('Cline task completed', {
      taskId: body.taskId,
      status: body.status
    });
    
    this.emit('taskCompleted', {
      taskId: body.taskId,
      status: body.status,
      result: body.result,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'task_completed' };
  }

  async handleErrorOccurred(body) {
    this.logger.error('Cline error occurred', {
      errorType: body.errorType,
      message: body.message
    });
    
    this.emit('errorOccurred', {
      errorType: body.errorType,
      message: body.message,
      context: body.context,
      timestamp: new Date()
    });
    
    return { status: 'processed', type: 'error_occurred' };
  }

  
  registerWebhookHandler(agentType, handler, secret = null, signatureHeader = null) {
    this.webhookHandlers.set(agentType, {
      handler,
      secret,
      signatureHeader
    });
    
    this.logger.info('Custom webhook handler registered', { agentType });
  }

  
  getWebhookStats() {
    return {
      registeredHandlers: Array.from(this.webhookHandlers.keys()),
      totalHandlers: this.webhookHandlers.size,
      uptime: process.uptime()
    };
  }

  
  getHealthStatus() {
    return {
      status: 'healthy',
      handlers: this.webhookHandlers.size,
      registeredAgents: Array.from(this.webhookHandlers.keys()),
      uptime: process.uptime()
    };
  }
}
