import { Octokit } from '@octokit/rest';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { logger } from '../../utils/logger';

export class GitHubConnector implements ConnectorInterface {
  private octokit?: Octokit;

  async validate(credentials: { accessToken: string }): Promise<boolean> {
    try {
      const octokit = new Octokit({ auth: credentials.accessToken });
      await octokit.rest.users.getAuthenticated();
      return true;
    } catch (error) {
      logger.error('GitHub credential validation failed:', error);
      return false;
    }
  }

  async connect(credentials: { accessToken: string }, config: any): Promise<boolean> {
    try {
      this.octokit = new Octokit({ auth: credentials.accessToken });
      await this.octokit.rest.users.getAuthenticated();
      return true;
    } catch (error) {
      logger.error('GitHub connection failed:', error);
      return false;
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