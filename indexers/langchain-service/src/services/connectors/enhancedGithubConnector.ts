import { Octokit } from '@octokit/rest';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { getGitHubAppAuth } from '../auth/githubAppAuth';
import { logger } from '../../utils/logger';

export interface GitHubAuthMethod {
  type: 'token' | 'oauth' | 'app';
  accessToken?: string;
  installationId?: string;
  userId?: string;
}

export class EnhancedGitHubConnector implements ConnectorInterface {
  private octokit?: Octokit;
  private authMethod?: GitHubAuthMethod;

  async validate(credentials: any): Promise<boolean> {
    try {
      const octokit = await this.createOctokitInstance(credentials);
      const response = await octokit.rest.users.getAuthenticated();
      
      logger.info('GitHub authentication validated:', {
        user: response.data.login,
        authType: credentials.authMethod?.type || 'token',
        scopes: response.headers['x-oauth-scopes']
      });
      
      return true;
    } catch (error: any) {
      logger.error('GitHub validation failed:', error.message);
      throw new Error(this.getErrorMessage(error));
    }
  }

  async connect(credentials: any, config: any): Promise<boolean> {
    try {
      this.octokit = await this.createOctokitInstance(credentials);
      this.authMethod = credentials.authMethod;
      
      const response = await this.octokit.rest.users.getAuthenticated();
      
      // Test repository access if specified
      if (config.repositories && config.repositories.length > 0) {
        await this.testRepositoryAccess(config.repositories[0]);
      }
      
      logger.info('GitHub connected successfully:', {
        user: response.data.login,
        authType: this.authMethod?.type || 'token'
      });
      
      return true;
    } catch (error: any) {
      logger.error('GitHub connection failed:', error.message);
      throw new Error(this.getErrorMessage(error));
    }
  }

  private async createOctokitInstance(credentials: any): Promise<Octokit> {
    const authMethod: GitHubAuthMethod = credentials.authMethod || { type: 'token' };
    
    switch (authMethod.type) {
      case 'token':
        if (!credentials.accessToken) {
          throw new Error('Access token is required for token authentication');
        }
        return new Octokit({ auth: credentials.accessToken });
        
      case 'oauth':
        if (!credentials.accessToken) {
          throw new Error('Access token is required for OAuth authentication');
        }
        return new Octokit({ auth: credentials.accessToken });
        
      case 'app':
        const githubApp = getGitHubAppAuth();
        if (!authMethod.installationId) {
          throw new Error('Installation ID is required for GitHub App authentication');
        }
        return await githubApp.getInstallationOctokit(authMethod.installationId);
        
      default:
        throw new Error(`Unsupported authentication method: ${authMethod.type}`);
    }
  }

  async sync(dataSource: DataSource): Promise<{ documents: any[], repositories: any[] }> {
    if (!this.octokit) {
      throw new Error('GitHub not connected');
    }

    const documents: any[] = [];
    const repositories: any[] = [];
    const { repositories: repoList, includeReadme, includeCode, fileExtensions } = dataSource.config;

    for (const repoName of repoList || []) {
      try {
        await this.syncRepository(repoName, documents, repositories, {
          includeReadme,
          includeCode,
          fileExtensions
        });
      } catch (error: any) {
        logger.error(`Failed to sync repository ${repoName}:`, error.message);
        // Continue with other repositories instead of failing completely
      }
    }

    return { documents, repositories };
  }

  private async syncRepository(
    repoName: string, 
    documents: any[], 
    repositories: any[], 
    options: any
  ): Promise<void> {
    if (!this.octokit) {
      throw new Error('GitHub not connected');
    }

    const [owner, repo] = repoName.split('/');
    
    // Get repository info
    const { data: repoData } = await this.octokit.rest.repos.get({ owner, repo });
    repositories.push(repoData);

    logger.info(`Syncing repository: ${repoName} (Private: ${repoData.private})`);

    // Get repository contents if requested
    if (options.includeCode) {
      const contents = await this.getRepositoryContents(
        owner, 
        repo, 
        '', 
        options.fileExtensions || []
      );
      
      for (const content of contents) {
        documents.push({
          id: `github-${repoName}-${content.path}`,
          title: content.name,
          content: content.content,
          metadata: {
            source: 'github',
            repository: repoName,
            path: content.path,
            type: content.type,
            size: content.size,
            sha: content.sha,
            url: content.html_url,
            lastModified: repoData.updated_at,
            private: repoData.private
          }
        });
      }
    }

    // Get README if requested
    if (options.includeReadme) {
      try {
        const { data: readme } = await this.octokit.rest.repos.getReadme({ owner, repo });
        const readmeContent = Buffer.from(readme.content, 'base64').toString('utf-8');
        
        documents.push({
          id: `github-${repoName}-readme`,
          title: `${repoName} README`,
          content: readmeContent,
          metadata: {
            source: 'github',
            repository: repoName,
            path: readme.path,
            type: 'readme',
            url: readme.html_url,
            private: repoData.private
          }
        });
      } catch (error) {
        logger.warn(`No README found for ${repoName}`);
      }
    }
  }

  private async getRepositoryContents(
    owner: string, 
    repo: string, 
    path: string = '', 
    fileExtensions: string[] = []
  ): Promise<any[]> {
    if (!this.octokit) {
      throw new Error('GitHub not connected');
    }

    const contents: any[] = [];
    
    try {
      const { data } = await this.octokit.rest.repos.getContent({ owner, repo, path });
      const items = Array.isArray(data) ? data : [data];
      
      for (const item of items) {
        if (item.type === 'file') {
          // Check file extension if filtering is enabled
          if (fileExtensions.length > 0) {
            const hasValidExtension = fileExtensions.some(ext => 
              item.name.toLowerCase().endsWith(ext.toLowerCase())
            );
            if (!hasValidExtension) continue;
          }
          
          // Skip binary files and large files
          if (item.size && item.size > 1000000) { // 1MB limit
            logger.warn(`Skipping large file: ${item.path} (${item.size} bytes)`);
            continue;
          }
          
          try {
            const { data: fileData } = await this.octokit.rest.repos.getContent({
              owner,
              repo,
              path: item.path
            });
            
            if ('content' in fileData && fileData.content) {
              const content = Buffer.from(fileData.content, 'base64').toString('utf-8');
              contents.push({
                ...item,
                content
              });
            }
          } catch (error) {
            logger.warn(`Failed to get content for ${item.path}:`, error);
          }
        } else if (item.type === 'dir') {
          // Recursively get directory contents (with depth limit)
          const subContents = await this.getRepositoryContents(owner, repo, item.path, fileExtensions);
          contents.push(...subContents);
        }
      }
    } catch (error: any) {
      if (error.status === 403) {
        throw new Error(`Access denied to repository contents. Your token may lack the required permissions for private repository: ${owner}/${repo}`);
      }
      throw error;
    }
    
    return contents;
  }

  private async testRepositoryAccess(repoName: string): Promise<void> {
    if (!this.octokit) {
      throw new Error('GitHub not connected');
    }

    try {
      const [owner, repo] = repoName.split('/');
      if (!owner || !repo) {
        throw new Error(`Invalid repository format: ${repoName}. Expected format: owner/repo`);
      }

      logger.info(`Testing access to repository: ${repoName}`);
      
      // Test repository access
      const repoData = await this.octokit.rest.repos.get({ owner, repo });
      logger.info(`Repository access successful:`, {
        repo: repoData.data.full_name,
        private: repoData.data.private,
        permissions: repoData.data.permissions
      });
      
      // Test contents access for private repos
      if (repoData.data.private) {
        try {
          await this.octokit.rest.repos.getContent({ owner, repo, path: '' });
          logger.info(`Private repository contents access verified: ${repoName}`);
        } catch (contentError: any) {
          if (contentError.status === 403) {
            throw new Error(`Access denied to private repository contents: ${repoName}. Your authentication method may lack the required permissions.`);
          }
          throw contentError;
        }
      }
      
    } catch (error: any) {
      this.handleRepositoryAccessError(error, repoName);
    }
  }

  private handleRepositoryAccessError(error: any, repoName: string): void {
    if (error.status === 404) {
      throw new Error(`Repository not found: ${repoName}. This could mean:\n• Repository doesn't exist\n• Repository is private and your authentication lacks access\n• Organization requires authentication approval\n• Repository name is misspelled`);
    } else if (error.status === 403) {
      const authType = this.authMethod?.type || 'token';
      let message = `Access denied to repository: ${repoName}.\n\n`;
      
      switch (authType) {
        case 'token':
          message += 'For personal access tokens:\n• Classic tokens need "repo" scope for private repos\n• Fine-grained tokens need repository access + Contents/Metadata permissions\n• Organization may require token approval';
          break;
        case 'app':
          message += 'For GitHub App authentication:\n• App must be installed on the organization/repository\n• App must have Contents and Metadata permissions\n• Installation must grant access to the repository';
          break;
        case 'oauth':
          message += 'For OAuth authentication:\n• User must have access to the repository\n• OAuth scope must include "repo" for private repositories';
          break;
      }
      
      throw new Error(message);
    } else if (error.status === 401) {
      throw new Error('Authentication failed. Please check your credentials.');
    } else {
      throw new Error(`Failed to access repository ${repoName}: ${error.message}`);
    }
  }

  private getErrorMessage(error: any): string {
    if (error.status === 401) {
      return 'Authentication failed. Please check your credentials and ensure they are valid.';
    } else if (error.status === 403) {
      return 'Access denied. Please check your permissions and authentication method.';
    } else if (error.status === 404) {
      return 'Resource not found. This may indicate insufficient permissions or incorrect configuration.';
    } else if (error.message?.includes('rate limit')) {
      return 'GitHub API rate limit exceeded. Please wait before trying again.';
    } else if (error.message) {
      return `GitHub API error: ${error.message}`;
    } else {
      return 'Unknown GitHub API error occurred.';
    }
  }
}