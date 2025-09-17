import { Octokit } from '@octokit/rest';
import { logger } from '../../utils/logger';

export interface CopilotUsage {
  total_seats: number;
  seats: Array<{
    assignee: {
      login: string;
      id: number;
      type: string;
    };
    assigning_team?: {
      name: string;
      slug: string;
    };
    pending_cancellation_date?: string;
    last_activity_at?: string;
    last_activity_editor?: string;
    created_at: string;
    updated_at: string;
  }>;
}

export interface CopilotSeatManagement {
  seats_created: number;
  seats_cancelled: number;
}

export class GitHubCopilotService {
  private octokit: Octokit;

  constructor(token: string) {
    this.octokit = new Octokit({
      auth: token,
      baseUrl: 'https://api.github.com'
    });
  }

  /**
   * Get Copilot billing information for an organization
   */
  async getCopilotBilling(org: string): Promise<any> {
    try {
      const response = await this.octokit.request('GET /orgs/{org}/copilot/billing', {
        org,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Retrieved Copilot billing for org: ${org}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get Copilot billing for ${org}:`, error);
      throw new Error(`Failed to get Copilot billing: ${error.message}`);
    }
  }

  /**
   * Get Copilot seat information for an organization
   */
  async getCopilotSeats(org: string, page = 1, per_page = 50): Promise<CopilotUsage> {
    try {
      const response = await this.octokit.request('GET /orgs/{org}/copilot/billing/seats', {
        org,
        page,
        per_page,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Retrieved ${response.data.seats?.length || 0} Copilot seats for org: ${org}`);
      return response.data as CopilotUsage;
    } catch (error: any) {
      logger.error(`Failed to get Copilot seats for ${org}:`, error);
      throw new Error(`Failed to get Copilot seats: ${error.message}`);
    }
  }

  /**
   * Add users to Copilot
   */
  async addCopilotSeats(org: string, usernames: string[]): Promise<CopilotSeatManagement> {
    try {
      const response = await this.octokit.request('POST /orgs/{org}/copilot/billing/selected_users', {
        org,
        selected_usernames: usernames,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Added ${usernames.length} users to Copilot in org: ${org}`);
      return response.data as CopilotSeatManagement;
    } catch (error: any) {
      logger.error(`Failed to add Copilot seats for ${org}:`, error);
      throw new Error(`Failed to add Copilot seats: ${error.message}`);
    }
  }

  /**
   * Remove users from Copilot
   */
  async removeCopilotSeats(org: string, usernames: string[]): Promise<CopilotSeatManagement> {
    try {
      const response = await this.octokit.request('DELETE /orgs/{org}/copilot/billing/selected_users', {
        org,
        selected_usernames: usernames,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Removed ${usernames.length} users from Copilot in org: ${org}`);
      return response.data as CopilotSeatManagement;
    } catch (error: any) {
      logger.error(`Failed to remove Copilot seats for ${org}:`, error);
      throw new Error(`Failed to remove Copilot seats: ${error.message}`);
    }
  }

  /**
   * Get Copilot usage metrics for an organization
   */
  async getCopilotUsageMetrics(org: string, since?: string, until?: string): Promise<any> {
    try {
      const params: any = {
        org,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      };

      if (since) params.since = since;
      if (until) params.until = until;

      const response = await this.octokit.request('GET /orgs/{org}/copilot/usage', params);
      
      logger.info(`Retrieved Copilot usage metrics for org: ${org}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get Copilot usage metrics for ${org}:`, error);
      throw new Error(`Failed to get Copilot usage metrics: ${error.message}`);
    }
  }

  /**
   * Get Copilot metrics for an enterprise
   */
  async getEnterpriseCopilotUsage(enterprise: string, since?: string, until?: string): Promise<any> {
    try {
      const params: any = {
        enterprise,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      };

      if (since) params.since = since;
      if (until) params.until = until;

      const response = await this.octokit.request('GET /enterprises/{enterprise}/copilot/usage', params);
      
      logger.info(`Retrieved Copilot usage metrics for enterprise: ${enterprise}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get Copilot usage metrics for enterprise ${enterprise}:`, error);
      throw new Error(`Failed to get enterprise Copilot usage metrics: ${error.message}`);
    }
  }

  /**
   * Get user's Copilot information
   */
  async getUserCopilotInfo(): Promise<any> {
    try {
      const response = await this.octokit.request('GET /user/copilot', {
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info('Retrieved user Copilot information');
      return response.data;
    } catch (error: any) {
      logger.error('Failed to get user Copilot info:', error);
      throw new Error(`Failed to get user Copilot info: ${error.message}`);
    }
  }

  /**
   * List Copilot enabled repositories for an organization
   */
  async getCopilotEnabledRepos(org: string, page = 1, per_page = 100): Promise<any> {
    try {
      const response = await this.octokit.request('GET /orgs/{org}/copilot/billing/selected_repos', {
        org,
        page,
        per_page,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Retrieved ${response.data.repositories?.length || 0} Copilot enabled repos for org: ${org}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to get Copilot enabled repos for ${org}:`, error);
      throw new Error(`Failed to get Copilot enabled repos: ${error.message}`);
    }
  }

  /**
   * Enable Copilot for specific repositories
   */
  async enableCopilotForRepos(org: string, repositoryNames: string[]): Promise<any> {
    try {
      const response = await this.octokit.request('PUT /orgs/{org}/copilot/billing/selected_repos', {
        org,
        selected_repository_names: repositoryNames,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Enabled Copilot for ${repositoryNames.length} repos in org: ${org}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to enable Copilot for repos in ${org}:`, error);
      throw new Error(`Failed to enable Copilot for repos: ${error.message}`);
    }
  }

  /**
   * Disable Copilot for specific repositories
   */
  async disableCopilotForRepos(org: string, repositoryNames: string[]): Promise<any> {
    try {
      const response = await this.octokit.request('DELETE /orgs/{org}/copilot/billing/selected_repos', {
        org,
        selected_repository_names: repositoryNames,
        headers: {
          'X-GitHub-Api-Version': '2022-11-28'
        }
      });
      
      logger.info(`Disabled Copilot for ${repositoryNames.length} repos in org: ${org}`);
      return response.data;
    } catch (error: any) {
      logger.error(`Failed to disable Copilot for repos in ${org}:`, error);
      throw new Error(`Failed to disable Copilot for repos: ${error.message}`);
    }
  }
}

export default GitHubCopilotService;