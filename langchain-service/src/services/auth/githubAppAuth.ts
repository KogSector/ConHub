import { App } from '@octokit/app';
import { OAuthApp } from '@octokit/oauth-app';
import { Octokit } from '@octokit/rest';
import { logger } from '../../utils/logger';

export interface GitHubAppConfig {
  appId: string;
  privateKey: string;
  installationId?: string;
  clientId?: string;
  clientSecret?: string;
}

export class GitHubAppAuthenticator {
  private app?: App;
  private oauthApp?: OAuthApp;
  private config: GitHubAppConfig;

  constructor(config: GitHubAppConfig) {
    this.config = config;
    
    // Initialize GitHub App if configured
    if (config.appId && config.privateKey) {
      this.app = new App({
        appId: config.appId,
        privateKey: config.privateKey,
      });
      logger.info('GitHub App initialized successfully');
    }
    
    // Initialize OAuth App if configured
    if (config.clientId && config.clientSecret) {
      this.oauthApp = new OAuthApp({
        clientId: config.clientId,
        clientSecret: config.clientSecret,
      });
      logger.info('GitHub OAuth App initialized successfully');
    }
  }

  /**
   * GitHub App Installation Authentication
   * Best for organization-wide access to private repositories
   */
  async getInstallationOctokit(installationId?: string): Promise<any> {
    if (!this.app) {
      throw new Error('GitHub App not configured. Please set GITHUB_APP_ID and GITHUB_APP_PRIVATE_KEY');
    }

    const id = installationId || this.config.installationId;
    if (!id) {
      throw new Error('Installation ID required for GitHub App authentication');
    }

    try {
      const installationOctokit = await this.app.getInstallationOctokit(parseInt(id));
      logger.info(`GitHub App authentication successful for installation: ${id}`);
      return installationOctokit;
    } catch (error: any) {
      logger.error('GitHub App authentication failed:', error);
      throw new Error(`GitHub App authentication failed: ${error.message}`);
    }
  }

  /**
   * Get OAuth URL for user authorization
   * Best for user-specific private repository access
   */
  getOAuthURL(state: string, scopes: string[] = ['repo', 'user:email']): string {
    if (!this.config.clientId) {
      throw new Error('GitHub OAuth not configured. Please set GITHUB_CLIENT_ID');
    }

    const params = new URLSearchParams({
      client_id: this.config.clientId,
      redirect_uri: `${process.env.FRONTEND_URL || 'http://localhost:3000'}/auth/github/callback`,
      scope: scopes.join(' '),
      state,
      allow_signup: 'true'
    });

    return `https://github.com/login/oauth/authorize?${params.toString()}`;
  }

  /**
   * Exchange OAuth code for access token using OAuthApp
   */
  async exchangeCodeForToken(code: string): Promise<string> {
    if (!this.oauthApp) {
      throw new Error('GitHub OAuth not configured. Please set GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET');
    }

    try {
      const result = await this.oauthApp.createToken({
        code,
      });

      logger.info('OAuth token exchange successful');
      return result.authentication.token;
    } catch (error: any) {
      logger.error('OAuth token exchange failed:', error);
      throw new Error(`OAuth authentication failed: ${error.message}`);
    }
  }

  /**
   * Get user information from OAuth token
   */
  async getUserFromToken(token: string): Promise<any> {
    try {
      const octokit = new Octokit({ auth: token });
      const { data: user } = await octokit.rest.users.getAuthenticated();
      
      logger.info('User retrieved from token:', user.login);
      return user;
    } catch (error: any) {
      logger.error('Failed to get user from token:', error);
      throw new Error(`Failed to validate token: ${error.message}`);
    }
  }

  /**
   * Get available installations for the GitHub App
   */
  async getInstallations(): Promise<any[]> {
    if (!this.app) {
      throw new Error('GitHub App not configured');
    }

    try {
      // Get app-level authentication
      const appOctokit = await this.app.getInstallationOctokit(undefined as any);
      const response = await appOctokit.request('GET /app/installations');
      return response.data;
    } catch (error: any) {
      logger.error('Failed to get GitHub App installations:', error);
      throw error;
    }
  }

  /**
   * Get repositories accessible to a specific installation
   */
  async getInstallationRepositories(installationId: string): Promise<any[]> {
    const octokit = await this.getInstallationOctokit(installationId);
    
    try {
      const { data } = await octokit.rest.apps.listReposAccessibleToInstallation({
        per_page: 100
      });
      
      return data.repositories;
    } catch (error: any) {
      logger.error('Failed to get installation repositories:', error);
      throw error;
    }
  }
}

// Singleton instance
let githubAppAuth: GitHubAppAuthenticator | null = null;

export function getGitHubAppAuth(): GitHubAppAuthenticator {
  if (!githubAppAuth) {
    const config: GitHubAppConfig = {
      appId: process.env.GITHUB_APP_ID || '',
      privateKey: process.env.GITHUB_APP_PRIVATE_KEY || '',
      installationId: process.env.GITHUB_APP_INSTALLATION_ID,
      clientId: process.env.GITHUB_CLIENT_ID,
      clientSecret: process.env.GITHUB_CLIENT_SECRET,
    };
    
    githubAppAuth = new GitHubAppAuthenticator(config);
  }
  
  return githubAppAuth;
}