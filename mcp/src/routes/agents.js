import express from 'express';
import Joi from 'joi';

const router = express.Router();

// Validation schemas
const registerAgentSchema = Joi.object({
  agentId: Joi.string().required(),
  config: Joi.object().optional()
});

const createSessionSchema = Joi.object({
  agentId: Joi.string().required(),
  userId: Joi.string().required(),
  config: Joi.object().optional()
});

const sendMessageSchema = Joi.object({
  sessionId: Joi.string().required(),
  message: Joi.string().required(),
  options: Joi.object().optional()
});

export default function agentRoutes(agentManager) {
  /**
   * Register a new AI agent
   * POST /api/agents/register
   */
  router.post('/register', async (req, res) => {
    try {
      const { error, value } = registerAgentSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { agentId, config } = value;
      const agent = await agentManager.registerAgent(agentId, config);

      res.status(201).json({
        success: true,
        agent: {
          id: agent.id,
          status: agent.status,
          capabilities: agent.capabilities,
          registeredAt: agent.registeredAt
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to register agent',
        message: error.message
      });
    }
  });

  /**
   * Connect to an AI agent
   * POST /api/agents/:agentId/connect
   */
  router.post('/:agentId/connect', async (req, res) => {
    try {
      const { agentId } = req.params;
      const connectionConfig = req.body;

      const agent = await agentManager.connectAgent(agentId, connectionConfig);

      res.json({
        success: true,
        agent: {
          id: agent.id,
          status: agent.status,
          capabilities: agent.capabilities,
          mcpConnection: agent.mcpConnection ? {
            id: agent.mcpConnection.id,
            status: agent.mcpConnection.status
          } : null
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to connect agent',
        message: error.message
      });
    }
  });

  /**
   * Disconnect an AI agent
   * POST /api/agents/:agentId/disconnect
   */
  router.post('/:agentId/disconnect', async (req, res) => {
    try {
      const { agentId } = req.params;
      await agentManager.disconnectAgent(agentId);

      res.json({
        success: true,
        message: 'Agent disconnected successfully'
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to disconnect agent',
        message: error.message
      });
    }
  });

  /**
   * Get all registered agents
   * GET /api/agents
   */
  router.get('/', async (req, res) => {
    try {
      const agents = agentManager.getAgents();

      res.json({
        success: true,
        agents: agents.map(agent => ({
          id: agent.id,
          status: agent.status,
          capabilities: agent.capabilities,
          registeredAt: agent.registeredAt,
          lastActivity: agent.lastActivity,
          sessions: agent.sessions.size,
          metrics: agent.metrics,
          mcpConnection: agent.mcpConnection ? {
            id: agent.mcpConnection.id,
            status: agent.mcpConnection.status
          } : null
        }))
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get agents',
        message: error.message
      });
    }
  });

  /**
   * Get specific agent
   * GET /api/agents/:agentId
   */
  router.get('/:agentId', async (req, res) => {
    try {
      const { agentId } = req.params;
      const agent = agentManager.getAgent(agentId);

      if (!agent) {
        return res.status(404).json({
          error: 'Agent not found'
        });
      }

      res.json({
        success: true,
        agent: {
          id: agent.id,
          status: agent.status,
          capabilities: agent.capabilities,
          registeredAt: agent.registeredAt,
          lastActivity: agent.lastActivity,
          sessions: agent.sessions.size,
          metrics: agent.metrics,
          config: agent.config,
          mcpConnection: agent.mcpConnection ? {
            id: agent.mcpConnection.id,
            status: agent.mcpConnection.status,
            capabilities: agent.mcpConnection.capabilities,
            serverInfo: agent.mcpConnection.serverInfo
          } : null
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get agent',
        message: error.message
      });
    }
  });

  /**
   * Create a new session with an agent
   * POST /api/agents/sessions
   */
  router.post('/sessions', async (req, res) => {
    try {
      const { error, value } = createSessionSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { agentId, userId, config } = value;
      const session = await agentManager.createSession(agentId, userId, config);

      res.status(201).json({
        success: true,
        session: {
          id: session.id,
          agentId: session.agentId,
          userId: session.userId,
          status: session.status,
          createdAt: session.createdAt,
          metrics: session.metrics
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to create session',
        message: error.message
      });
    }
  });

  /**
   * Send message to agent
   * POST /api/agents/sessions/message
   */
  router.post('/sessions/message', async (req, res) => {
    try {
      const { error, value } = sendMessageSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { sessionId, message, options } = value;
      const response = await agentManager.sendMessage(sessionId, message, options);

      res.json({
        success: true,
        response: {
          id: response.id,
          type: response.type,
          content: response.content,
          timestamp: response.timestamp
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to send message',
        message: error.message
      });
    }
  });

  /**
   * Get all sessions
   * GET /api/agents/sessions
   */
  router.get('/sessions', async (req, res) => {
    try {
      const { agentId, userId, status } = req.query;
      let sessions = agentManager.getSessions();

      // Filter sessions based on query parameters
      if (agentId) {
        sessions = sessions.filter(s => s.agentId === agentId);
      }
      if (userId) {
        sessions = sessions.filter(s => s.userId === userId);
      }
      if (status) {
        sessions = sessions.filter(s => s.status === status);
      }

      res.json({
        success: true,
        sessions: sessions.map(session => ({
          id: session.id,
          agentId: session.agentId,
          userId: session.userId,
          status: session.status,
          createdAt: session.createdAt,
          lastActivity: session.lastActivity,
          messageCount: session.messages.length,
          metrics: session.metrics
        }))
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get sessions',
        message: error.message
      });
    }
  });

  /**
   * Get specific session
   * GET /api/agents/sessions/:sessionId
   */
  router.get('/sessions/:sessionId', async (req, res) => {
    try {
      const { sessionId } = req.params;
      const session = agentManager.getSession(sessionId);

      if (!session) {
        return res.status(404).json({
          error: 'Session not found'
        });
      }

      res.json({
        success: true,
        session: {
          id: session.id,
          agentId: session.agentId,
          userId: session.userId,
          status: session.status,
          createdAt: session.createdAt,
          lastActivity: session.lastActivity,
          messages: session.messages,
          context: session.context,
          metrics: session.metrics
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get session',
        message: error.message
      });
    }
  });

  /**
   * Close a session
   * DELETE /api/agents/sessions/:sessionId
   */
  router.delete('/sessions/:sessionId', async (req, res) => {
    try {
      const { sessionId } = req.params;
      await agentManager.closeSession(sessionId);

      res.json({
        success: true,
        message: 'Session closed successfully'
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to close session',
        message: error.message
      });
    }
  });

  /**
   * Get agent capabilities
   * GET /api/agents/:agentId/capabilities
   */
  router.get('/:agentId/capabilities', async (req, res) => {
    try {
      const { agentId } = req.params;
      const agent = agentManager.getAgent(agentId);

      if (!agent) {
        return res.status(404).json({
          error: 'Agent not found'
        });
      }

      res.json({
        success: true,
        capabilities: agent.capabilities,
        mcpCapabilities: agent.mcpConnection?.capabilities || null
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get agent capabilities',
        message: error.message
      });
    }
  });

  /**
   * Get agent metrics
   * GET /api/agents/:agentId/metrics
   */
  router.get('/:agentId/metrics', async (req, res) => {
    try {
      const { agentId } = req.params;
      const agent = agentManager.getAgent(agentId);

      if (!agent) {
        return res.status(404).json({
          error: 'Agent not found'
        });
      }

      res.json({
        success: true,
        metrics: {
          ...agent.metrics,
          activeSessions: agent.sessions.size,
          status: agent.status,
          lastActivity: agent.lastActivity,
          uptime: agent.registeredAt ? Date.now() - agent.registeredAt.getTime() : 0
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get agent metrics',
        message: error.message
      });
    }
  });

  /**
   * Get service health and metrics
   * GET /api/agents/health
   */
  router.get('/health', async (req, res) => {
    try {
      const health = agentManager.getHealthStatus();
      res.json(health);
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get health status',
        message: error.message
      });
    }
  });

  /**
   * Get detailed service metrics
   * GET /api/agents/metrics
   */
  router.get('/metrics', async (req, res) => {
    try {
      const metrics = agentManager.getMetrics();
      res.json({
        success: true,
        metrics
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get metrics',
        message: error.message
      });
    }
  });

  return router;
}
