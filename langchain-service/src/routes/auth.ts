import express from 'express';
import { getGitHubAppAuth } from '../services/auth/githubAppAuth';
import { logger } from '../utils/logger';

const router = express.Router();

// Get GitHub OAuth URL for user authentication
router.get('/github/url', async (req, res) => {
  try {
    const { state } = req.query;
    
    if (!state) {
      return res.status(400).json({ error: 'State parameter is required' });
    }
    
    const githubAuth = getGitHubAppAuth();
    const oauthUrl = githubAuth.getOAuthURL(state as string, ['repo', 'user:email']);
    
    res.json({
      success: true,
      oauthUrl
    });
  } catch (error: any) {
    logger.error('Failed to generate OAuth URL:', error);
    res.status(500).json({ 
      error: 'Failed to generate OAuth URL',
      details: error.message 
    });
  }
});

// Handle OAuth callback and exchange code for token
router.post('/github/callback', async (req, res) => {
  try {
    const { code, state } = req.body;
    
    if (!code) {
      return res.status(400).json({ error: 'Authorization code is required' });
    }
    
    const githubAuth = getGitHubAppAuth();
    const accessToken = await githubAuth.exchangeCodeForToken(code);
    
    res.json({
      success: true,
      accessToken,
      authMethod: {
        type: 'oauth',
        token: accessToken
      }
    });
  } catch (error: any) {
    logger.error('OAuth callback failed:', error);
    res.status(500).json({ 
      error: 'OAuth authentication failed',
      details: error.message 
    });
  }
});

// Get available GitHub App installations
router.get('/github/installations', async (req, res) => {
  try {
    const githubAuth = getGitHubAppAuth();
    const installations = await githubAuth.getInstallations();
    
    res.json({
      success: true,
      installations: installations.map(installation => ({
        id: installation.id,
        account: {
          login: installation.account.login,
          type: installation.account.type,
          avatarUrl: installation.account.avatar_url
        },
        repositorySelection: installation.repository_selection,
        permissions: installation.permissions
      }))
    });
  } catch (error: any) {
    logger.error('Failed to get installations:', error);
    res.status(500).json({ 
      error: 'Failed to get GitHub App installations',
      details: error.message 
    });
  }
});

// Get repositories for a specific installation
router.get('/github/installations/:installationId/repositories', async (req, res) => {
  try {
    const { installationId } = req.params;
    
    const githubAuth = getGitHubAppAuth();
    const repositories = await githubAuth.getInstallationRepositories(installationId);
    
    res.json({
      success: true,
      repositories: repositories.map(repo => ({
        id: repo.id,
        name: repo.name,
        fullName: repo.full_name,
        private: repo.private,
        url: repo.html_url,
        description: repo.description,
        language: repo.language,
        updatedAt: repo.updated_at
      }))
    });
  } catch (error: any) {
    logger.error('Failed to get installation repositories:', error);
    res.status(500).json({ 
      error: 'Failed to get installation repositories',
      details: error.message 
    });
  }
});

export default router;