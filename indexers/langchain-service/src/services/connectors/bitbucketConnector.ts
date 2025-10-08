import axios from 'axios';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { logger } from '../../utils/logger';

export class BitBucketConnector implements ConnectorInterface {
  private apiClient?: any;
  private credentials?: { username: string; appPassword: string };

  async validate(credentials: { username: string; appPassword: string }): Promise<boolean> {
    try {
      const response = await axios.get('https://api.bitbucket.org/2.0/user', {
        auth: {
          username: credentials.username,
          password: credentials.appPassword
        }
      });
      return response.status === 200;
    } catch (error) {
      logger.error('BitBucket credential validation failed:', error);
      return false;
    }
  }

  async connect(credentials: { username: string; appPassword: string }, config: any): Promise<boolean> {
    try {
      this.credentials = credentials;
      this.apiClient = axios.create({
        baseURL: 'https://api.bitbucket.org/2.0',
        auth: {
          username: credentials.username,
          password: credentials.appPassword
        }
      });
      
      // Test connection
      await this.apiClient.get('/user');
      return true;
    } catch (error) {
      logger.error('BitBucket connection failed:', error);
      return false;
    }
  }

  async sync(dataSource: DataSource): Promise<{ documents: any[], repositories: any[] }> {
    if (!this.apiClient || !this.credentials) {
      throw new Error('BitBucket not connected');
    }

    const documents: any[] = [];
    const repositories: any[] = [];
    const { repositories: repoList, includeReadme, includeCode, fileExtensions } = dataSource.config;

    for (const repoName of repoList || []) {
      try {
        const [workspace, repo] = repoName.split('/');
        
        // Get repository info
        const { data: repoData } = await this.apiClient.get(`/repositories/${workspace}/${repo}`);
        repositories.push(repoData);

        // Get repository contents
        const contents = await this.getRepositoryContents(workspace, repo, '', includeCode, fileExtensions);
        
        for (const content of contents) {
          documents.push({
            id: `bitbucket-${repoName}-${content.path}`,
            title: content.name,
            content: content.content,
            metadata: {
              source: 'bitbucket',
              repository: repoName,
              path: content.path,
              type: content.type,
              size: content.size,
              commit: content.commit,
              url: content.links?.self?.href,
              lastModified: repoData.updated_on
            }
          });
        }

        // Get README if requested
        if (includeReadme) {
          try {
            const { data: readme } = await this.apiClient.get(`/repositories/${workspace}/${repo}/src/main/README.md`);
            
            documents.push({
              id: `bitbucket-${repoName}-readme`,
              title: `${repoName} README`,
              content: readme,
              metadata: {
                source: 'bitbucket',
                repository: repoName,
                path: 'README.md',
                type: 'readme'
              }
            });
          } catch (error) {
            logger.warn(`No README found for ${repoName}`);
          }
        }

        // Get issues and PRs
        try {
          const { data: issues } = await this.apiClient.get(`/repositories/${workspace}/${repo}/issues`);
          
          for (const issue of issues.values || []) {
            documents.push({
              id: `bitbucket-${repoName}-issue-${issue.id}`,
              title: issue.title,
              content: issue.content?.raw || '',
              metadata: {
                source: 'bitbucket',
                repository: repoName,
                type: 'issue',
                id: issue.id,
                state: issue.state,
                priority: issue.priority,
                kind: issue.kind,
                author: issue.reporter?.display_name,
                createdAt: issue.created_on,
                updatedAt: issue.updated_on
              }
            });
          }
        } catch (error) {
          logger.warn(`Could not fetch issues for ${repoName}:`, error);
        }

        // Get pull requests
        try {
          const { data: pullRequests } = await this.apiClient.get(`/repositories/${workspace}/${repo}/pullrequests`);
          
          for (const pr of pullRequests.values || []) {
            documents.push({
              id: `bitbucket-${repoName}-pr-${pr.id}`,
              title: pr.title,
              content: pr.description || '',
              metadata: {
                source: 'bitbucket',
                repository: repoName,
                type: 'pull_request',
                id: pr.id,
                state: pr.state,
                author: pr.author?.display_name,
                source_branch: pr.source?.branch?.name,
                destination_branch: pr.destination?.branch?.name,
                createdAt: pr.created_on,
                updatedAt: pr.updated_on
              }
            });
          }
        } catch (error) {
          logger.warn(`Could not fetch pull requests for ${repoName}:`, error);
        }

      } catch (error) {
        logger.error(`Error syncing repository ${repoName}:`, error);
      }
    }

    return { documents, repositories };
  }

  private async getRepositoryContents(
    workspace: string,
    repo: string,
    path: string = '',
    includeCode: boolean = true,
    fileExtensions: string[] = []
  ): Promise<any[]> {
    if (!this.apiClient) return [];

    const contents: any[] = [];
    
    try {
      const url = path 
        ? `/repositories/${workspace}/${repo}/src/main/${path}`
        : `/repositories/${workspace}/${repo}/src/main/`;
        
      const { data } = await this.apiClient.get(url);
      const items = data.values || [];

      for (const item of items) {
        if (item.type === 'commit_file') {
          // Check file extension filter
          if (fileExtensions.length > 0) {
            const ext = '.' + item.path.split('.').pop()?.toLowerCase();
            if (!fileExtensions.includes(ext)) continue;
          }

          // Skip large files
          if (item.size && item.size > 1024 * 1024) continue;

          try {
            const { data: fileContent } = await this.apiClient.get(`/repositories/${workspace}/${repo}/src/main/${item.path}`);
            
            contents.push({
              ...item,
              content: typeof fileContent === 'string' ? fileContent : JSON.stringify(fileContent),
              name: item.path.split('/').pop()
            });
          } catch (error) {
            logger.warn(`Could not fetch content for ${item.path}:`, error);
          }
        } else if (item.type === 'commit_directory' && includeCode) {
          // Recursively get directory contents
          const subContents = await this.getRepositoryContents(workspace, repo, item.path, includeCode, fileExtensions);
          contents.push(...subContents);
        }
      }
    } catch (error) {
      logger.error(`Error getting repository contents for ${workspace}/${repo}:`, error);
    }

    return contents;
  }
}