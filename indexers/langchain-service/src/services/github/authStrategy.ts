import { GitHubIntegrationService } from './integrationService';
import { getGitHubAppAuth } from '../auth/githubAppAuth';
import { logger } from '../../utils/logger';

export enum UserTier {
  FREE = 'free',
  PERSONAL = 'personal',
  TEAM = 'team',
  ENTERPRISE = 'enterprise'
}

export enum AuthMethod {
  PERSONAL_ACCESS_TOKEN = 'pat',
  GITHUB_APP = 'github_app',
  OAUTH = 'oauth'
}

export interface AuthConfig {
  method: AuthMethod;
  tier: UserTier;
  token?: string;
  installationId?: string;
  repositoryType: 'personal' | 'organization';
  organizationName?: string;
}

export class GitHubAuthStrategy {
  
  /**
   * Determine the best authentication method based on user tier and repository type
   */
  static getRecommendedAuthMethod(
    userTier: UserTier, 
    repositoryType: 'personal' | 'organization',
    organizationName?: string
  ): AuthMethod {
    
    // For organization repositories
    if (repositoryType === 'organization') {
      switch (userTier) {
        case UserTier.FREE:
          // Free tier users can only use PAT for org repos they have personal access to
          logger.warn(`Free tier user accessing org repo. PAT may have limited functionality.`);
          return AuthMethod.PERSONAL_ACCESS_TOKEN;
          
        case UserTier.PERSONAL:
          // Personal tier can use PAT but GitHub App is recommended for better org integration
          logger.info(`Personal tier user: GitHub App recommended for org ${organizationName}`);
          return AuthMethod.GITHUB_APP;
          
        case UserTier.TEAM:
        case UserTier.ENTERPRISE:
          // Business tiers should use GitHub Apps for org repositories
          logger.info(`Business tier user: Using GitHub App for org ${organizationName}`);
          return AuthMethod.GITHUB_APP;
      }
    }
    
    // For personal repositories
    if (repositoryType === 'personal') {
      switch (userTier) {
        case UserTier.FREE:
        case UserTier.PERSONAL:
          // Individual users can use PAT for their personal repos
          return AuthMethod.PERSONAL_ACCESS_TOKEN;
          
        case UserTier.TEAM:
        case UserTier.ENTERPRISE:
          // Business users can choose, but PAT is fine for personal repos
          return AuthMethod.PERSONAL_ACCESS_TOKEN;
      }
    }
    
    // Default fallback
    return AuthMethod.PERSONAL_ACCESS_TOKEN;
  }

  /**
   * Create GitHub integration service based on auth strategy
   */
  static async createIntegrationService(config: AuthConfig): Promise<GitHubIntegrationService> {
    logger.info(`Creating GitHub integration with ${config.method} for ${config.tier} tier user`);
    
    switch (config.method) {
      case AuthMethod.PERSONAL_ACCESS_TOKEN:
        if (!config.token) {
          throw new Error('Personal Access Token is required for PAT authentication');
        }
        return new GitHubIntegrationService({ token: config.token });
        
      case AuthMethod.GITHUB_APP:
        if (!config.installationId) {
          throw new Error('Installation ID is required for GitHub App authentication');
        }
        return await GitHubIntegrationService.createWithAppAuth(config.installationId);
        
      case AuthMethod.OAUTH:
        // OAuth implementation would go here
        throw new Error('OAuth authentication not yet implemented');
        
      default:
        throw new Error(`Unsupported authentication method: ${config.method}`);
    }
  }

  /**
   * Validate authentication configuration
   */
  static validateAuthConfig(config: AuthConfig): { valid: boolean; message?: string } {
    
    // Check for required fields based on auth method
    switch (config.method) {
      case AuthMethod.PERSONAL_ACCESS_TOKEN:
        if (!config.token) {
          return { valid: false, message: 'Personal Access Token is required' };
        }
        if (config.token && !config.token.startsWith('ghp_') && !config.token.startsWith('github_pat_')) {
          return { valid: false, message: 'Invalid token format. Expected classic (ghp_) or fine-grained (github_pat_) token' };
        }
        break;
        
      case AuthMethod.GITHUB_APP:
        if (!config.installationId) {
          return { valid: false, message: 'GitHub App Installation ID is required' };
        }
        break;
    }

    // Warn about tier-specific limitations
    if (config.tier === UserTier.FREE && config.repositoryType === 'organization') {
      return { 
        valid: true, 
        message: 'Warning: Free tier users may have limited access to organization repositories. Consider upgrading for GitHub App integration.' 
      };
    }

    return { valid: true };
  }

  /**
   * Get user guidance based on their tier and repository type
   */
  static getUserGuidance(
    userTier: UserTier,
    repositoryType: 'personal' | 'organization',
    organizationName?: string
  ): string {
    
    const recommendedMethod = this.getRecommendedAuthMethod(userTier, repositoryType, organizationName);
    
    const guidance = {
      [AuthMethod.PERSONAL_ACCESS_TOKEN]: {
        title: 'Personal Access Token (PAT)',
        description: 'Use your GitHub personal access token',
        steps: [
          '1. Go to GitHub Settings → Developer settings → Personal access tokens',
          '2. Generate new token (classic) with "repo" scope for private repos',
          '3. Copy the token and paste it in the connection form',
          '4. Your token will have the same access level as your user account'
        ]
      },
      [AuthMethod.GITHUB_APP]: {
        title: 'GitHub App Integration',
        description: 'Install ConHub GitHub App for enhanced organization access',
        steps: [
          '1. Install the ConHub GitHub App in your organization',
          '2. Grant repository access permissions during installation',
          '3. The app will have fine-grained access only to selected repositories',
          '4. Better audit trail and organization-level permissions'
        ]
      }
    };

    const method = guidance[recommendedMethod];
    let message = `**Recommended: ${method.title}**\n\n${method.description}\n\n`;
    message += method.steps.join('\n');

    // Add tier-specific notes
    if (userTier === UserTier.FREE && repositoryType === 'organization') {
      message += '\n\n⚠️ **Note**: Free tier users have limited organization access. Personal Access Tokens work for repositories you have personal access to.';
    }

    if ([UserTier.TEAM, UserTier.ENTERPRISE].includes(userTier) && repositoryType === 'organization') {
      message += '\n\n✨ **Enterprise Feature**: GitHub Apps provide enhanced security, audit trails, and fine-grained permissions for your organization.';
    }

    return message;
  }
}

export default GitHubAuthStrategy;