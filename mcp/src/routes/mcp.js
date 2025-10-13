import express from 'express';
import Joi from 'joi';

const router = express.Router();


const initConnectionSchema = Joi.object({
  agentId: Joi.string().required(),
  config: Joi.object().optional()
});

const listResourcesSchema = Joi.object({
  connectionId: Joi.string().required()
});

const readResourceSchema = Joi.object({
  connectionId: Joi.string().required(),
  uri: Joi.string().required()
});

const callToolSchema = Joi.object({
  connectionId: Joi.string().required(),
  toolName: Joi.string().required(),
  arguments: Joi.object().optional()
});

export default function mcpRoutes(mcpService) {
  
  router.post('/connections', async (req, res) => {
    try {
      const { error, value } = initConnectionSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { agentId, config } = value;
      const connection = await mcpService.initializeConnection(agentId, config);

      res.status(201).json({
        success: true,
        connection: {
          id: connection.id,
          agentId: connection.agentId,
          status: connection.status,
          capabilities: connection.capabilities,
          serverInfo: connection.serverInfo,
          createdAt: connection.createdAt
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to initialize MCP connection',
        message: error.message
      });
    }
  });

  
  router.get('/connections', async (req, res) => {
    try {
      const connections = mcpService.getConnections();
      
      res.json({
        success: true,
        connections: connections.map(conn => ({
          id: conn.id,
          agentId: conn.agentId,
          status: conn.status,
          capabilities: conn.capabilities,
          serverInfo: conn.serverInfo,
          createdAt: conn.createdAt,
          lastActivity: conn.lastActivity
        }))
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get connections',
        message: error.message
      });
    }
  });

  
  router.get('/connections/:connectionId', async (req, res) => {
    try {
      const { connectionId } = req.params;
      const connection = mcpService.getConnection(connectionId);

      if (!connection) {
        return res.status(404).json({
          error: 'Connection not found'
        });
      }

      res.json({
        success: true,
        connection: {
          id: connection.id,
          agentId: connection.agentId,
          status: connection.status,
          capabilities: connection.capabilities,
          serverInfo: connection.serverInfo,
          createdAt: connection.createdAt,
          lastActivity: connection.lastActivity
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get connection',
        message: error.message
      });
    }
  });

  
  router.get('/connections/:connectionId/resources', async (req, res) => {
    try {
      const { connectionId } = req.params;
      const resources = await mcpService.listResources(connectionId);

      res.json({
        success: true,
        resources
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to list resources',
        message: error.message
      });
    }
  });

  
  router.post('/resources/read', async (req, res) => {
    try {
      const { error, value } = readResourceSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { connectionId, uri } = value;
      const resource = await mcpService.readResource(connectionId, uri);

      res.json({
        success: true,
        resource
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to read resource',
        message: error.message
      });
    }
  });

  
  router.post('/resources/subscribe', async (req, res) => {
    try {
      const { error, value } = readResourceSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { connectionId, uri } = value;
      const subscribed = await mcpService.subscribeToResource(connectionId, uri);

      res.json({
        success: true,
        subscribed
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to subscribe to resource',
        message: error.message
      });
    }
  });

  
  router.get('/connections/:connectionId/tools', async (req, res) => {
    try {
      const { connectionId } = req.params;
      const tools = await mcpService.listTools(connectionId);

      res.json({
        success: true,
        tools
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to list tools',
        message: error.message
      });
    }
  });

  
  router.post('/tools/call', async (req, res) => {
    try {
      const { error, value } = callToolSchema.validate(req.body);
      if (error) {
        return res.status(400).json({
          error: 'Validation failed',
          details: error.details
        });
      }

      const { connectionId, toolName, arguments: args } = value;
      const result = await mcpService.callTool(connectionId, toolName, args);

      res.json({
        success: true,
        result
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to call tool',
        message: error.message
      });
    }
  });

  
  router.delete('/connections/:connectionId', async (req, res) => {
    try {
      const { connectionId } = req.params;
      await mcpService.closeConnection(connectionId);

      res.json({
        success: true,
        message: 'Connection closed successfully'
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to close connection',
        message: error.message
      });
    }
  });

  
  router.get('/health', async (req, res) => {
    try {
      const health = mcpService.getHealthStatus();
      res.json(health);
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get health status',
        message: error.message
      });
    }
  });

  return router;
}
