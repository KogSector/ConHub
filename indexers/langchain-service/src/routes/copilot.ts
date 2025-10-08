import { Router } from 'express';
import { GitHubCopilotService } from '../services/github/copilotService';
import { logger } from '../utils/logger';

const router = Router();

/**
 * Initialize GitHub Copilot service with authentication
 */
async function initializeCopilotService(req: any): Promise<GitHubCopilotService> {
  const authHeader = req.headers.authorization;
  
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw new Error('Authorization token required');
  }

  const token = authHeader.substring(7);
  return new GitHubCopilotService(token);
}

/**
 * Get Copilot billing information for an organization
 */
router.get('/billing/:org', async (req, res) => {
  try {
    const { org } = req.params;
    const copilotService = await initializeCopilotService(req);
    
    const billing = await copilotService.getCopilotBilling(org);
    
    res.json({
      success: true,
      data: billing
    });
  } catch (error: any) {
    logger.error('Error getting Copilot billing:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get Copilot seats for an organization
 */
router.get('/seats/:org', async (req, res) => {
  try {
    const { org } = req.params;
    const { page = 1, per_page = 50 } = req.query;
    const copilotService = await initializeCopilotService(req);
    
    const seats = await copilotService.getCopilotSeats(
      org, 
      parseInt(page as string), 
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: seats
    });
  } catch (error: any) {
    logger.error('Error getting Copilot seats:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Add users to Copilot
 */
router.post('/seats/:org/add', async (req, res) => {
  try {
    const { org } = req.params;
    const { usernames } = req.body;
    
    if (!usernames || !Array.isArray(usernames)) {
      return res.status(400).json({
        success: false,
        error: 'usernames array is required'
      });
    }

    const copilotService = await initializeCopilotService(req);
    const result = await copilotService.addCopilotSeats(org, usernames);
    
    res.json({
      success: true,
      data: result
    });
  } catch (error: any) {
    logger.error('Error adding Copilot seats:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Remove users from Copilot
 */
router.delete('/seats/:org/remove', async (req, res) => {
  try {
    const { org } = req.params;
    const { usernames } = req.body;
    
    if (!usernames || !Array.isArray(usernames)) {
      return res.status(400).json({
        success: false,
        error: 'usernames array is required'
      });
    }

    const copilotService = await initializeCopilotService(req);
    const result = await copilotService.removeCopilotSeats(org, usernames);
    
    res.json({
      success: true,
      data: result
    });
  } catch (error: any) {
    logger.error('Error removing Copilot seats:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get Copilot usage metrics for an organization
 */
router.get('/usage/:org', async (req, res) => {
  try {
    const { org } = req.params;
    const { since, until } = req.query;
    const copilotService = await initializeCopilotService(req);
    
    const usage = await copilotService.getCopilotUsageMetrics(
      org, 
      since as string, 
      until as string
    );
    
    res.json({
      success: true,
      data: usage
    });
  } catch (error: any) {
    logger.error('Error getting Copilot usage metrics:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get enterprise Copilot usage metrics
 */
router.get('/usage/enterprise/:enterprise', async (req, res) => {
  try {
    const { enterprise } = req.params;
    const { since, until } = req.query;
    const copilotService = await initializeCopilotService(req);
    
    const usage = await copilotService.getEnterpriseCopilotUsage(
      enterprise, 
      since as string, 
      until as string
    );
    
    res.json({
      success: true,
      data: usage
    });
  } catch (error: any) {
    logger.error('Error getting enterprise Copilot usage metrics:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get user's Copilot information
 */
router.get('/user/info', async (req, res) => {
  try {
    const copilotService = await initializeCopilotService(req);
    const userInfo = await copilotService.getUserCopilotInfo();
    
    res.json({
      success: true,
      data: userInfo
    });
  } catch (error: any) {
    logger.error('Error getting user Copilot info:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get Copilot enabled repositories for an organization
 */
router.get('/repos/:org', async (req, res) => {
  try {
    const { org } = req.params;
    const { page = 1, per_page = 100 } = req.query;
    const copilotService = await initializeCopilotService(req);
    
    const repos = await copilotService.getCopilotEnabledRepos(
      org, 
      parseInt(page as string), 
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: repos
    });
  } catch (error: any) {
    logger.error('Error getting Copilot enabled repos:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Enable Copilot for specific repositories
 */
router.post('/repos/:org/enable', async (req, res) => {
  try {
    const { org } = req.params;
    const { repository_names } = req.body;
    
    if (!repository_names || !Array.isArray(repository_names)) {
      return res.status(400).json({
        success: false,
        error: 'repository_names array is required'
      });
    }

    const copilotService = await initializeCopilotService(req);
    const result = await copilotService.enableCopilotForRepos(org, repository_names);
    
    res.json({
      success: true,
      data: result
    });
  } catch (error: any) {
    logger.error('Error enabling Copilot for repos:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Disable Copilot for specific repositories
 */
router.delete('/repos/:org/disable', async (req, res) => {
  try {
    const { org } = req.params;
    const { repository_names } = req.body;
    
    if (!repository_names || !Array.isArray(repository_names)) {
      return res.status(400).json({
        success: false,
        error: 'repository_names array is required'
      });
    }

    const copilotService = await initializeCopilotService(req);
    const result = await copilotService.disableCopilotForRepos(org, repository_names);
    
    res.json({
      success: true,
      data: result
    });
  } catch (error: any) {
    logger.error('Error disabling Copilot for repos:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

export default router;