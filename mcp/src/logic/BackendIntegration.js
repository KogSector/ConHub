

import axios from 'axios';
import { EventEmitter } from 'events';
import { v4 as uuidv4 } from 'uuid';


export class BackendApiClient {
  constructor(baseUrl, logger) {
    this.baseUrl = baseUrl || process.env.BACKEND_URL || 'http://localhost:3001';
    this.logger = logger;
    
    this.client = axios.create({
      baseURL: this.baseUrl,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'ConHub-MCP-Service/1.0.0'
      }
    });

    this.setupInterceptors();
  }

  setupInterceptors() {
    
    this.client.interceptors.request.use(
      (config) => {
        this.logger.debug('Backend API request', {
          method: config.method,
          url: config.url,
          data: config.data
        });
        return config;
      },
      (error) => {
        this.logger.error('Backend API request error', { error: error.message });
        return Promise.reject(error);
      }
    );

    
    this.client.interceptors.response.use(
      (response) => {
        this.logger.debug('Backend API response', {
          status: response.status,
          url: response.config.url
        });
        return response;
      },
      (error) => {
        this.logger.error('Backend API response error', {
          status: error.response?.status,
          url: error.config?.url,
          message: error.message
        });
        return Promise.reject(error);
      }
    );
  }

  
  async authenticateUser(token) {
    try {
      const response = await this.client.post('/api/auth/verify', {}, {
        headers: { Authorization: `Bearer ${token}` }
      });
      return response.data;
    } catch (error) {
      throw new Error(`Authentication failed: ${error.message}`);
    }
  }

  async getUserProfile(token) {
    try {
      const response = await this.client.get('/api/auth/profile', {
        headers: { Authorization: `Bearer ${token}` }
      });
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get user profile: ${error.message}`);
    }
  }

  
  async triggerIndexing(indexingRequest) {
    try {
      const response = await this.client.post('/api/indexing/repository', indexingRequest);
      return response.data;
    } catch (error) {
      throw new Error(`Indexing request failed: ${error.message}`);
    }
  }

  async getIndexingStatus(requestId) {
    try {
      const response = await this.client.get(`/api/indexing/status/${requestId}`);
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get indexing status: ${error.message}`);
    }
  }

  
  async searchVectorDb(query, filters = {}) {
    try {
      const response = await this.client.post('/api/search/vector', {
        query,
        filters,
        limit: 10
      });
      return response.data;
    } catch (error) {
      throw new Error(`Vector search failed: ${error.message}`);
    }
  }

  async storeEmbedding(embedding, metadata) {
    try {
      const response = await this.client.post('/api/embeddings/store', {
        embedding,
        metadata
      });
      return response.data;
    } catch (error) {
      throw new Error(`Failed to store embedding: ${error.message}`);
    }
  }

  
  async getRepositories(userId) {
    try {
      const response = await this.client.get(`/api/repositories?userId=${userId}`);
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get repositories: ${error.message}`);
    }
  }

  async getRepositoryContent(repoId, path = '') {
    try {
      const response = await this.client.get(`/api/repositories/${repoId}/content`, {
        params: { path }
      });
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get repository content: ${error.message}`);
    }
  }

  
  async getUserSubscription(userId) {
    try {
      const response = await this.client.get(`/api/billing/subscription/${userId}`);
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get user subscription: ${error.message}`);
    }
  }

  async checkUsageLimits(userId, feature) {
    try {
      const response = await this.client.get(`/api/billing/usage/${userId}/${feature}`);
      return response.data;
    } catch (error) {
      throw new Error(`Failed to check usage limits: ${error.message}`);
    }
  }

  
  async healthCheck() {
    try {
      const response = await this.client.get('/health');
      return response.data;
    } catch (error) {
      throw new Error(`Backend health check failed: ${error.message}`);
    }
  }
}


export class ContextSynchronizer extends EventEmitter {
  constructor(backendClient, logger) {
    super();
    
    this.backendClient = backendClient;
    this.logger = logger;
    this.contextCache = new Map();
    this.syncInterval = null;
    
    this.startPeriodicSync();
  }

  async syncAgentContext(agentId, context) {
    try {
      const contextData = {
        agentId,
        context,
        timestamp: new Date().toISOString(),
        version: 1
      };

      
      this.contextCache.set(agentId, contextData);

      
      await this.backendClient.client.post('/api/agents/context', contextData);

      this.logger.debug('Agent context synchronized', { agentId });
      this.emit('contextSynced', { agentId, context });

    } catch (error) {
      this.logger.error('Failed to sync agent context', {
        agentId,
        error: error.message
      });
      this.emit('syncError', { agentId, error });
    }
  }

  async getAgentContext(agentId) {
    try {
      
      if (this.contextCache.has(agentId)) {
        return this.contextCache.get(agentId).context;
      }

      
      const response = await this.backendClient.client.get(`/api/agents/context/${agentId}`);
      const contextData = response.data;

      
      this.contextCache.set(agentId, contextData);

      return contextData.context;

    } catch (error) {
      this.logger.error('Failed to get agent context', {
        agentId,
        error: error.message
      });
      return null;
    }
  }

  startPeriodicSync() {
    this.syncInterval = setInterval(() => {
      this.syncAllContexts();
    }, 60000); 
  }

  async syncAllContexts() {
    const contexts = Array.from(this.contextCache.entries());
    
    for (const [agentId, contextData] of contexts) {
      try {
        await this.backendClient.client.post('/api/agents/context', contextData);
      } catch (error) {
        this.logger.error('Failed to sync context during periodic sync', {
          agentId,
          error: error.message
        });
      }
    }
  }

  stop() {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
    }
  }
}


export class ResourceProvider {
  constructor(backendClient, logger) {
    this.backendClient = backendClient;
    this.logger = logger;
    this.resourceCache = new Map();
    this.cacheTimeout = 300000; 
  }

  async getCodebaseResources(userId, repositoryId) {
    const cacheKey = `codebase:${userId}:${repositoryId}`;
    
    
    if (this.resourceCache.has(cacheKey)) {
      const cached = this.resourceCache.get(cacheKey);
      if (Date.now() - cached.timestamp < this.cacheTimeout) {
        return cached.data;
      }
    }

    try {
      const resources = await this.backendClient.getRepositoryContent(repositoryId);
      
      
      this.resourceCache.set(cacheKey, {
        data: resources,
        timestamp: Date.now()
      });

      return resources;

    } catch (error) {
      this.logger.error('Failed to get codebase resources', {
        userId,
        repositoryId,
        error: error.message
      });
      throw error;
    }
  }

  async getDocumentationResources(userId, query) {
    try {
      const searchResults = await this.backendClient.searchVectorDb(query, {
        type: 'documentation',
        userId
      });

      return searchResults.results || [];

    } catch (error) {
      this.logger.error('Failed to get documentation resources', {
        userId,
        query,
        error: error.message
      });
      throw error;
    }
  }

  async getContextualResources(userId, context) {
    try {
      
      const searchQuery = this.extractSearchQuery(context);
      const searchResults = await this.backendClient.searchVectorDb(searchQuery, {
        userId,
        contextual: true
      });

      return searchResults.results || [];

    } catch (error) {
      this.logger.error('Failed to get contextual resources', {
        userId,
        error: error.message
      });
      throw error;
    }
  }

  extractSearchQuery(context) {
    
    if (typeof context === 'string') {
      return context;
    }

    if (context.messages && Array.isArray(context.messages)) {
      return context.messages
        .filter(msg => msg.type === 'user')
        .map(msg => msg.content)
        .join(' ');
    }

    return JSON.stringify(context);
  }

  clearCache() {
    this.resourceCache.clear();
  }
}


export class UsageTracker extends EventEmitter {
  constructor(backendClient, logger) {
    super();
    
    this.backendClient = backendClient;
    this.logger = logger;
    this.usageBuffer = new Map();
    this.flushInterval = null;
    
    this.startPeriodicFlush();
  }

  trackAgentUsage(userId, agentType, action, metadata = {}) {
    const key = `${userId}:${agentType}`;
    
    if (!this.usageBuffer.has(key)) {
      this.usageBuffer.set(key, {
        userId,
        agentType,
        actions: [],
        totalActions: 0,
        startTime: new Date()
      });
    }

    const usage = this.usageBuffer.get(key);
    usage.actions.push({
      action,
      metadata,
      timestamp: new Date()
    });
    usage.totalActions++;

    this.emit('usageTracked', { userId, agentType, action, metadata });
  }

  async checkUsageLimits(userId, agentType) {
    try {
      const limits = await this.backendClient.checkUsageLimits(userId, agentType);
      return limits;
    } catch (error) {
      this.logger.error('Failed to check usage limits', {
        userId,
        agentType,
        error: error.message
      });
      return { allowed: true, remaining: 1000 }; 
    }
  }

  startPeriodicFlush() {
    this.flushInterval = setInterval(() => {
      this.flushUsageData();
    }, 30000); 
  }

  async flushUsageData() {
    const usageEntries = Array.from(this.usageBuffer.entries());
    
    for (const [key, usage] of usageEntries) {
      try {
        
        await this.backendClient.client.post('/api/billing/usage', usage);
        
        
        this.usageBuffer.delete(key);
        
      } catch (error) {
        this.logger.error('Failed to flush usage data', {
          key,
          error: error.message
        });
      }
    }
  }

  stop() {
    if (this.flushInterval) {
      clearInterval(this.flushInterval);
      this.flushInterval = null;
    }
    
    
    this.flushUsageData();
  }
}


export class BackendIntegrationManager extends EventEmitter {
  constructor(logger) {
    super();
    
    this.logger = logger;
    this.backendClient = new BackendApiClient(null, logger);
    this.contextSynchronizer = new ContextSynchronizer(this.backendClient, logger);
    this.resourceProvider = new ResourceProvider(this.backendClient, logger);
    this.usageTracker = new UsageTracker(this.backendClient, logger);
    
    this.setupEventHandlers();
  }

  setupEventHandlers() {
    this.contextSynchronizer.on('contextSynced', (event) => {
      this.emit('contextSynced', event);
    });

    this.contextSynchronizer.on('syncError', (event) => {
      this.emit('syncError', event);
    });

    this.usageTracker.on('usageTracked', (event) => {
      this.emit('usageTracked', event);
    });
  }

  
  async authenticateUser(token) {
    return await this.backendClient.authenticateUser(token);
  }

  async getUserProfile(token) {
    return await this.backendClient.getUserProfile(token);
  }

  
  async syncAgentContext(agentId, context) {
    return await this.contextSynchronizer.syncAgentContext(agentId, context);
  }

  async getAgentContext(agentId) {
    return await this.contextSynchronizer.getAgentContext(agentId);
  }

  
  async getResourcesForAgent(userId, agentType, context = {}) {
    const resources = [];

    try {
      
      const contextualResources = await this.resourceProvider.getContextualResources(userId, context);
      resources.push(...contextualResources);

      
      if (context.needsDocumentation) {
        const docResources = await this.resourceProvider.getDocumentationResources(
          userId, 
          context.query || ''
        );
        resources.push(...docResources);
      }

      
      if (context.repositoryId) {
        const codeResources = await this.resourceProvider.getCodebaseResources(
          userId, 
          context.repositoryId
        );
        resources.push(...codeResources);
      }

      return resources;

    } catch (error) {
      this.logger.error('Failed to get resources for agent', {
        userId,
        agentType,
        error: error.message
      });
      return [];
    }
  }

  
  trackUsage(userId, agentType, action, metadata = {}) {
    this.usageTracker.trackAgentUsage(userId, agentType, action, metadata);
  }

  async checkUsageLimits(userId, agentType) {
    return await this.usageTracker.checkUsageLimits(userId, agentType);
  }

  
  async triggerIndexing(indexingRequest) {
    return await this.backendClient.triggerIndexing(indexingRequest);
  }

  async getIndexingStatus(requestId) {
    return await this.backendClient.getIndexingStatus(requestId);
  }

  
  async getBackendHealth() {
    return await this.backendClient.healthCheck();
  }

  getIntegrationStatus() {
    return {
      backendConnected: true, 
      contextSyncActive: this.contextSynchronizer.syncInterval !== null,
      usageTrackingActive: this.usageTracker.flushInterval !== null,
      resourceCacheSize: this.resourceProvider.resourceCache.size,
      usageBufferSize: this.usageTracker.usageBuffer.size
    };
  }

  
  shutdown() {
    this.contextSynchronizer.stop();
    this.usageTracker.stop();
    this.resourceProvider.clearCache();
    
    this.logger.info('Backend integration manager shut down');
  }
}
