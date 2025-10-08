import express from 'express';
import { aiAgentService } from '../services/aiAgentService';
import { logger } from '../utils/logger';

const router = express.Router();

// Get all AI agents
router.get('/', async (req, res) => {
  try {
    const agents = await aiAgentService.getAgents();
    res.json({ success: true, agents });
  } catch (error) {
    logger.error('Error getting AI agents:', error);
    res.status(500).json({ error: 'Failed to get AI agents' });
  }
});

// Register new AI agent
router.post('/register', async (req, res) => {
  try {
    const { name, type, config, credentials } = req.body;
    const agent = await aiAgentService.registerAgent({
      name,
      type,
      status: 'connected',
      config,
      credentials
    });
    res.json({ success: true, agent });
  } catch (error) {
    logger.error('Error registering AI agent:', error);
    res.status(500).json({ error: 'Failed to register AI agent' });
  }
});

// Query AI agent
router.post('/query', async (req, res) => {
  try {
    const { query, context, agentId, includeContext } = req.body;
    const response = await aiAgentService.queryAgent({
      query,
      context,
      agentId,
      includeContext
    });
    res.json({ success: true, ...response });
  } catch (error) {
    logger.error('Error querying AI agent:', error);
    res.status(500).json({ error: 'Failed to query AI agent' });
  }
});

export default router;