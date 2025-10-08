import { GitHubCopilotService } from './copilotService';
import { logger } from '../../utils/logger';
import { Octokit } from '@octokit/rest';

export interface GitHubIntegrationConfig {
  token?: string;
  appId?: string;
  privateKey?: string;
  installationId?: string;
  clientId?: string;
  clientSecret?: string;
}

export interface RepositoryInfo {
  id: number;
  name: string;
  full_name: string;
  private: boolean;
  html_url: string;
  description?: string;
  language?: string;
  stargazers_count: number;
  forks_count: number;
  updated_at: string;
  topics: string[];
}

export interface OrganizationInfo {
  login: string;
  id: number;
  avatar_url: string;
  description?: string;
  name?: string;
  company?: string;
  blog?: string;
  location?: string;
  public_repos: number;
  public_gists: number;
  followers: number;
  following: number;
  created_at: string;
  updated_at: string;
}

export class GitHubIntegrationService {
  private octokit: Octokit;
  private copilotService?: GitHubCopilotService;
  private config: GitHubIntegrationConfig;

  constructor(config: GitHubIntegrationConfig) {
    this.config = config;
    
    if (config.token) {
      this.octokit = new Octokit({
        auth: config.token,
        baseUrl: 'https://api.github.com'
      });
      this.copilotService = new GitHubCopilotService(config.token);
    } else {
      throw new Error('GitHub token is required for integration');
    }
  }

  /**
   * Initialize with GitHub App authentication
   */
  static async createWithAppAuth(installationId?: string): Promise<GitHubIntegrationService> {
    try {
      // Lazy load to avoid startup issues
      const { getGitHubAppAuth } = await import('../auth/githubAppAuth');
      const appAuth = getGitHubAppAuth();
      const octokit = await appAuth.getInstallationOctokit(installationId);
      
      // Extract token from the authenticated Octokit instance
      const token = (octokit as any).auth?.token || (octokit as any).auth;
      
      return new GitHubIntegrationService({ 
        token,
        installationId 
      });
    } catch (error: any) {
      logger.error('Failed to create GitHub integration with App auth:', error);
      throw error;
    }
  }

  /**
   * Get authenticated user information
   */
  async getCurrentUser(): Promise<any> {
    try {
      const response = await this.octokit.rest.users.getAuthenticated();
      logger.info('Retrieved current user information');
      return response.data;
    } catch (error: any) {
      logger.error('Failed to get current user:', error);
      throw new Error(`Failed to get current user: ${error.message}`);
    }
  }

  /**
   * Get user repositories
   */
  async getUserRepositories(username?: string, page = 1, per_page = 100): Promise<RepositoryInfo[]> {
    try {
      let response;
      
      if (username) {
        response = await this.octokit.rest.repos.listForUser({
          username,
          page,
          per_page,
          sort: 'updated',
          direction: 'desc'
        });
      } else {
        response = await this.octokit.rest.repos.listForAuthenticatedUser({
          page,
          per_page,
          sort: 'updated',
          direction: 'desc'
        });
      }

      logger.info(`Retrieved ${response.data.length} repositories`);
      return response.data as RepositoryInfo[];
    } catch (error: any) {
      logger.error('Failed to get user repositories:', error);
      throw new Error(`Failed to get repositories: ${error.message}`);
    }
  }

  /**
   * Get organization repositories
   */
  async getOrganizationRepositories(org: string, page = 1, per_page = 100): Promise<RepositoryInfo[]> {
    try {
      const response = await this.octokit.rest.repos.listForOrg({
        org,
        page,
        per_page,
        sort: 'updated',
        direction: 'desc'
      });

      logger.info(`Retrieved ${response.data.length} repositories for org: ${org}`);
      return response.data as RepositoryInfo[];
    } catch (error: any) {
      logger.error(`Failed to get organization repositories for ${org}:`, error);
      throw new Error(`Failed to get organization repositories: ${error.message}`);
    }
  }

  /**
   * Get user organizations
   */
  async getUserOrganizations(): Promise<any[]> {
    try {
      const response = await this.octokit.rest.orgs.listForAuthenticatedUser({
        per_page: 100
      });

      logger.info(`Retrieved ${response.data.length} organizations`);
      return response.data;
    } catch (error: any) {
      logger.error('Failed to get user organizations:', error);
      throw new Error(`Failed to get organizations: ${error.message}`);
    }
  }

  /**
   * Get organization information
   */
  async getOrganization(org: string): Promise<OrganizationInfo> {
    try {
      const response = await this.octokit.rest.orgs.get({ org });
      logger.info(`Retrieved organization info for: ${org}`);
      return response.data as OrganizationInfo;
    } catch (error: any) {
      logger.error(`Failed to get organization ${org}:`, error);
      throw new Error(`Failed to get organization: ${error.message}`);
    }
  }

  /**
   * Get repository content
   */
  async getRepositoryContent(owner: string, repo: string, path = '', ref?: string): Promise<any> {
    try {
      const params: any = { owner, repo, path };
      if (ref) params.ref = ref;

      const response = await this.octokit.rest.repos.getContent(params);
      logger.info(`Retrieved content for ${owner}/${repo}:${path}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get repository content for ${owner}/${repo}:`, error);
      throw new Error(`Failed to get repository content: ${error.message}`);
    }
  }

  /**
   * Search repositories
   */
  async searchRepositories(query: string, page = 1, per_page = 100): Promise<any> {
    try {
      const response = await this.octokit.rest.search.repos({
        q: query,
        page,
        per_page,
        sort: 'updated',
        order: 'desc'
      });

      logger.info(`Found ${response.data.total_count} repositories for query: ${query}`);
      return response.data;
    } catch (error: any) {
      logger.error('Failed to search repositories:', error);
      throw new Error(`Failed to search repositories: ${error.message}`);
    }
  }

  /**
   * Get repository commits
   */
  async getRepositoryCommits(owner: string, repo: string, page = 1, per_page = 100): Promise<any[]> {
    try {
      const response = await this.octokit.rest.repos.listCommits({
        owner,
        repo,
        page,
        per_page
      });

      logger.info(`Retrieved ${response.data.length} commits for ${owner}/${repo}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get commits for ${owner}/${repo}:`, error);
      throw new Error(`Failed to get repository commits: ${error.message}`);
    }
  }

  /**
   * Get repository issues
   */
  async getRepositoryIssues(owner: string, repo: string, state: 'open' | 'closed' | 'all' = 'open', page = 1, per_page = 100): Promise<any[]> {
    try {
      const response = await this.octokit.rest.issues.listForRepo({
        owner,
        repo,
        state,
        page,
        per_page,
        sort: 'updated',
        direction: 'desc'
      });

      logger.info(`Retrieved ${response.data.length} issues for ${owner}/${repo}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get issues for ${owner}/${repo}:`, error);
      throw new Error(`Failed to get repository issues: ${error.message}`);
    }
  }

  /**
   * Get repository pull requests
   */
  async getRepositoryPullRequests(owner: string, repo: string, state: 'open' | 'closed' | 'all' = 'open', page = 1, per_page = 100): Promise<any[]> {
    try {
      const response = await this.octokit.rest.pulls.list({
        owner,
        repo,
        state,
        page,
        per_page,
        sort: 'updated',
        direction: 'desc'
      });

      logger.info(`Retrieved ${response.data.length} pull requests for ${owner}/${repo}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get pull requests for ${owner}/${repo}:`, error);
      throw new Error(`Failed to get repository pull requests: ${error.message}`);
    }
  }

  /**
   * Get Copilot service instance
   */
  getCopilotService(): GitHubCopilotService {
    if (!this.copilotService) {
      throw new Error('Copilot service not available. Ensure a valid token is provided.');
    }
    return this.copilotService;
  }

  /**
   * Get comprehensive repository analytics
   */
  async getRepositoryAnalytics(owner: string, repo: string): Promise<any> {
    try {
      const [repoInfo, commits, issues, pullRequests] = await Promise.all([
        this.octokit.rest.repos.get({ owner, repo }),
        this.getRepositoryCommits(owner, repo, 1, 10),
        this.getRepositoryIssues(owner, repo, 'all', 1, 10),
        this.getRepositoryPullRequests(owner, repo, 'all', 1, 10)
      ]);

      const analytics = {
        repository: repoInfo.data,
        recent_commits: commits,
        recent_issues: issues,
        recent_pull_requests: pullRequests,
        metrics: {
          total_commits: commits.length,
          open_issues: issues.filter((issue: any) => issue.state === 'open').length,
          closed_issues: issues.filter((issue: any) => issue.state === 'closed').length,
          open_prs: pullRequests.filter((pr: any) => pr.state === 'open').length,
          closed_prs: pullRequests.filter((pr: any) => pr.state === 'closed').length
        }
      };

      logger.info(`Generated analytics for ${owner}/${repo}`);
      return analytics;
    } catch (error: any) {
      logger.error(`Failed to generate analytics for ${owner}/${repo}:`, error);
      throw new Error(`Failed to generate repository analytics: ${error.message}`);
    }
  }
}

export default GitHubIntegrationService;