import { Router } from 'express';
import { GitHubIntegrationService } from '../services/github/integrationService';
import { logger } from '../utils/logger';

const router = Router();

/**
 * Initialize GitHub integration service with authentication
 */
async function initializeGitHubService(req: any): Promise<GitHubIntegrationService> {
  const authHeader = req.headers.authorization;
  
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw new Error('Authorization token required');
  }

  const token = authHeader.substring(7);
  return new GitHubIntegrationService({ token });
}

/**
 * Get current authenticated user
 */
router.get('/user', async (req, res) => {
  try {
    const githubService = await initializeGitHubService(req);
    const user = await githubService.getCurrentUser();
    
    res.json({
      success: true,
      data: user
    });
  } catch (error: any) {
    logger.error('Error getting current user:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get user repositories
 */
router.get('/repositories', async (req, res) => {
  try {
    const { username, page = 1, per_page = 100 } = req.query;
    const githubService = await initializeGitHubService(req);
    
    const repositories = await githubService.getUserRepositories(
      username as string,
      parseInt(page as string),
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: repositories
    });
  } catch (error: any) {
    logger.error('Error getting repositories:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get organization repositories
 */
router.get('/organizations/:org/repositories', async (req, res) => {
  try {
    const { org } = req.params;
    const { page = 1, per_page = 100 } = req.query;
    const githubService = await initializeGitHubService(req);
    
    const repositories = await githubService.getOrganizationRepositories(
      org,
      parseInt(page as string),
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: repositories
    });
  } catch (error: any) {
    logger.error('Error getting organization repositories:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get user organizations
 */
router.get('/organizations', async (req, res) => {
  try {
    const githubService = await initializeGitHubService(req);
    const organizations = await githubService.getUserOrganizations();
    
    res.json({
      success: true,
      data: organizations
    });
  } catch (error: any) {
    logger.error('Error getting organizations:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get organization details
 */
router.get('/organizations/:org', async (req, res) => {
  try {
    const { org } = req.params;
    const githubService = await initializeGitHubService(req);
    
    const organization = await githubService.getOrganization(org);
    
    res.json({
      success: true,
      data: organization
    });
  } catch (error: any) {
    logger.error('Error getting organization:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get repository content
 */
router.get('/repositories/:owner/:repo/contents', async (req, res) => {
  try {
    const { owner, repo } = req.params;
    const { path = '', ref } = req.query;
    const githubService = await initializeGitHubService(req);
    
    const content = await githubService.getRepositoryContent(
      owner,
      repo,
      path as string,
      ref as string
    );
    
    res.json({
      success: true,
      data: content
    });
  } catch (error: any) {
    logger.error('Error getting repository content:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Search repositories
 */
router.get('/search/repositories', async (req, res) => {
  try {
    const { q, page = 1, per_page = 100 } = req.query;
    
    if (!q) {
      return res.status(400).json({
        success: false,
        error: 'Query parameter "q" is required'
      });
    }

    const githubService = await initializeGitHubService(req);
    const results = await githubService.searchRepositories(
      q as string,
      parseInt(page as string),
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: results
    });
  } catch (error: any) {
    logger.error('Error searching repositories:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get repository commits
 */
router.get('/repositories/:owner/:repo/commits', async (req, res) => {
  try {
    const { owner, repo } = req.params;
    const { page = 1, per_page = 100 } = req.query;
    const githubService = await initializeGitHubService(req);
    
    const commits = await githubService.getRepositoryCommits(
      owner,
      repo,
      parseInt(page as string),
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: commits
    });
  } catch (error: any) {
    logger.error('Error getting repository commits:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get repository issues
 */
router.get('/repositories/:owner/:repo/issues', async (req, res) => {
  try {
    const { owner, repo } = req.params;
    const { state = 'open', page = 1, per_page = 100 } = req.query;
    const githubService = await initializeGitHubService(req);
    
    const issues = await githubService.getRepositoryIssues(
      owner,
      repo,
      state as 'open' | 'closed' | 'all',
      parseInt(page as string),
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: issues
    });
  } catch (error: any) {
    logger.error('Error getting repository issues:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get repository pull requests
 */
router.get('/repositories/:owner/:repo/pulls', async (req, res) => {
  try {
    const { owner, repo } = req.params;
    const { state = 'open', page = 1, per_page = 100 } = req.query;
    const githubService = await initializeGitHubService(req);
    
    const pullRequests = await githubService.getRepositoryPullRequests(
      owner,
      repo,
      state as 'open' | 'closed' | 'all',
      parseInt(page as string),
      parseInt(per_page as string)
    );
    
    res.json({
      success: true,
      data: pullRequests
    });
  } catch (error: any) {
    logger.error('Error getting repository pull requests:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get comprehensive repository analytics
 */
router.get('/repositories/:owner/:repo/analytics', async (req, res) => {
  try {
    const { owner, repo } = req.params;
    const githubService = await initializeGitHubService(req);
    
    const analytics = await githubService.getRepositoryAnalytics(owner, repo);
    
    res.json({
      success: true,
      data: analytics
    });
  } catch (error: any) {
    logger.error('Error getting repository analytics:', error);
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

/**
 * Get Copilot service endpoints
 */
router.get('/copilot/user/info', async (req, res) => {
  try {
    const githubService = await initializeGitHubService(req);
    const copilotService = githubService.getCopilotService();
    
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
 * Health check for GitHub integration
 */
router.get('/health', async (req, res) => {
  try {
    const githubService = await initializeGitHubService(req);
    const user = await githubService.getCurrentUser();
    
    res.json({
      success: true,
      message: 'GitHub integration is healthy',
      user: {
        login: user.login,
        id: user.id,
        type: user.type
      }
    });
  } catch (error: any) {
    logger.error('GitHub integration health check failed:', error);
    res.status(500).json({
      success: false,
      error: 'GitHub integration is not healthy',
      details: error.message
    });
  }
});

export default router;