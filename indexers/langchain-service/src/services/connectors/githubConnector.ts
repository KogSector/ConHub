import { Octokit } from '@octokit/rest';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { logger } from '../../utils/logger';

export class GitHubConnector implements ConnectorInterface {
  private octokit?: Octokit;

  async validate(credentials: { accessToken: string }): Promise<boolean> {
    try {
      if (!credentials.accessToken) {
        logger.error('GitHub validation failed: No access token provided');
        throw new Error('GitHub access token is required');
      }

      const octokit = new Octokit({ auth: credentials.accessToken });
      const response = await octokit.rest.users.getAuthenticated();
      
      // Check token scopes for debugging
      const scopes = response.headers['x-oauth-scopes'];
      logger.info('GitHub token validated successfully:', {
        user: response.data.login,
        scopes: scopes,
        tokenType: credentials.accessToken.startsWith('ghp_') ? 'classic' : 
                  credentials.accessToken.startsWith('github_pat_') ? 'fine-grained' : 'unknown'
      });
      
      // Validate scopes for private repo access
      if (scopes && !scopes.includes('repo') && !scopes.includes('public_repo')) {
        logger.warn('Token may lack required scopes for repository access:', scopes);
      }
      
      return true;
    } catch (error: any) {
      const errorMsg = this.getErrorMessage(error);
      logger.error('GitHub credential validation failed:', errorMsg);
      throw new Error(errorMsg);
    }
  }

  async connect(credentials: { accessToken: string }, config: any): Promise<boolean> {
    try {
      if (!credentials.accessToken) {
        throw new Error('GitHub access token is required');
      }

      this.octokit = new Octokit({ auth: credentials.accessToken });
      const response = await this.octokit.rest.users.getAuthenticated();
      
      // Test repository access if repositories are specified
      if (config.repositories && config.repositories.length > 0) {
        await this.testRepositoryAccess(config.repositories[0]);
      }
      
      logger.info('GitHub connected successfully for user:', response.data.login);
      return true;
    } catch (error: any) {
      const errorMsg = this.getErrorMessage(error);
      logger.error('GitHub connection failed:', errorMsg);
      throw new Error(errorMsg);
    }
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
      
      // First try to get basic repo info
      const repoData = await this.octokit.rest.repos.get({ owner, repo });
      logger.info(`Repository access successful. Repo: ${repoData.data.full_name}, Private: ${repoData.data.private}, Permissions: ${JSON.stringify(repoData.data.permissions)}`);
      
      // Test if we can read contents (crucial for private repos)
      try {
        await this.octokit.rest.repos.getContent({
          owner,
          repo,
          path: ''
        });
        logger.info(`Repository contents access verified for: ${repoName}`);
      } catch (contentError: any) {
        logger.warn(`Contents access test failed for ${repoName}:`, contentError.message);
        if (contentError.status === 403) {
          throw new Error(`Access denied to repository contents: ${repoName}. Your token may need 'repo' scope for private repositories or 'Contents' permission for fine-grained tokens.`);
        }
      }
      
    } catch (error: any) {
      logger.error(`Repository access test failed for ${repoName}:`, error);
      
      if (error.status === 404) {
        throw new Error(`Repository not found: ${repoName}. This could mean:\n• Repository doesn't exist\n• Repository is private and token lacks access\n• Organization requires token approval\n• Repository name is misspelled`);
      } else if (error.status === 403) {
        throw new Error(`Access denied to repository: ${repoName}. For private repositories:\n• Classic tokens need 'repo' scope (not just 'public_repo')\n• Fine-grained tokens need repository access + Contents/Metadata permissions\n• Organization may require token approval`);
      } else if (error.status === 401) {
        throw new Error('GitHub token is invalid or expired. Please check your access token.');
      } else {
        throw new Error(`Failed to access repository ${repoName}: ${error.message}`);
      }
    }
  }

  private getErrorMessage(error: any): string {
    if (error.status === 401) {
      return 'GitHub token is invalid or expired. Please check your access token and ensure it has not been revoked.';
    } else if (error.status === 403) {
      return 'GitHub token lacks required permissions. For public repositories, use a token with "public_repo" scope. For private repositories, use "repo" scope.';
    } else if (error.status === 404) {
      return 'GitHub API endpoint not found. This may indicate an issue with the token or API access.';
    } else if (error.message?.includes('rate limit')) {
      return 'GitHub API rate limit exceeded. Please wait before trying again.';
    } else if (error.message) {
      return `GitHub API error: ${error.message}`;
    } else {
      return 'Unknown GitHub API error occurred.';
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
        const [owner, repo] = repoName.split('/');
        
        // Get repository info
        const { data: repoData } = await this.octokit.rest.repos.get({ owner, repo });
        repositories.push(repoData);

        // Get repository contents
        const contents = await this.getRepositoryContents(owner, repo, '', includeCode, fileExtensions);
        
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
              lastModified: repoData.updated_at
            }
          });
        }

        // Get README if requested
        if (includeReadme) {
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
                url: readme.html_url
              }
            });
          } catch (error) {
            logger.warn(`No README found for ${repoName}`);
          }
        }

        // Get issues and PRs
        const issues = await this.octokit.rest.issues.listForRepo({
          owner,
          repo,
          state: 'all',
          per_page: 100
        });

        for (const issue of issues.data) {
          documents.push({
            id: `github-${repoName}-issue-${issue.number}`,
            title: issue.title,
            content: issue.body || '',
            metadata: {
              source: 'github',
              repository: repoName,
              type: issue.pull_request ? 'pull_request' : 'issue',
              number: issue.number,
              state: issue.state,
              author: issue.user?.login,
              url: issue.html_url,
              createdAt: issue.created_at,
              updatedAt: issue.updated_at
            }
          });
        }

      } catch (error) {
        logger.error(`Error syncing repository ${repoName}:`, error);
      }
    }

    return { documents, repositories };
  }

  private async getRepositoryContents(
    owner: string, 
    repo: string, 
    path: string = '', 
    includeCode: boolean = true,
    fileExtensions: string[] = []
  ): Promise<any[]> {
    if (!this.octokit) return [];

    const contents: any[] = [];
    
    try {
      const { data } = await this.octokit.rest.repos.getContent({ owner, repo, path });
      const items = Array.isArray(data) ? data : [data];

      for (const item of items) {
        if (item.type === 'file') {
          // Check file extension filter
          if (fileExtensions.length > 0) {
            const ext = '.' + item.name.split('.').pop()?.toLowerCase();
            if (!fileExtensions.includes(ext)) continue;
          }

          // Skip binary files and large files
          if (item.size && item.size > 1024 * 1024) continue; // Skip files > 1MB

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
            logger.warn(`Could not fetch content for ${item.path}:`, error);
          }
        } else if (item.type === 'dir' && includeCode) {
          // Recursively get directory contents
          const subContents = await this.getRepositoryContents(owner, repo, item.path, includeCode, fileExtensions);
          contents.push(...subContents);
        }
      }
    } catch (error) {
      logger.error(`Error getting repository contents for ${owner}/${repo}:`, error);
    }

    return contents;
  }
}