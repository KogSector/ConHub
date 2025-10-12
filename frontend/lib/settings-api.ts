const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

export class SettingsAPI {
  private static async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<{ success: boolean; data?: T; error?: string }> {
    try {
      const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        ...options,
      });

      const result = await response.json();
      return result;
    } catch (error) {
      return {
        success: false,
        error: 'Network error occurred',
      };
    }
  }

  // Profile Settings
  static async getSettings(userId: string) {
    return this.request(`/api/settings/${userId}`);
  }

  static async updateSettings(userId: string, settings: any) {
    return this.request(`/api/settings/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // API Tokens
  static async getApiTokens(userId: string) {
    return this.request(`/api/settings/${userId}/api-tokens`);
  }

  static async createApiToken(userId: string, tokenData: { name: string; permissions: string[] }) {
    return this.request(`/api/settings/${userId}/api-tokens`, {
      method: 'POST',
      body: JSON.stringify(tokenData),
    });
  }

  static async deleteApiToken(userId: string, tokenId: string) {
    return this.request(`/api/settings/${userId}/api-tokens/${tokenId}`, {
      method: 'DELETE',
    });
  }

  // Webhooks
  static async getWebhooks(userId: string) {
    return this.request(`/api/settings/${userId}/webhooks`);
  }

  static async createWebhook(userId: string, webhookData: { name: string; url: string; events: string[] }) {
    return this.request(`/api/settings/${userId}/webhooks`, {
      method: 'POST',
      body: JSON.stringify(webhookData),
    });
  }

  static async deleteWebhook(userId: string, webhookId: string) {
    return this.request(`/api/settings/${userId}/webhooks/${webhookId}`, {
      method: 'DELETE',
    });
  }

  // Team Management
  static async getTeamMembers(userId: string) {
    return this.request(`/api/settings/${userId}/team`);
  }

  static async inviteTeamMember(userId: string, memberData: { email: string; role: string }) {
    return this.request(`/api/settings/${userId}/team`, {
      method: 'POST',
      body: JSON.stringify(memberData),
    });
  }

  static async removeTeamMember(userId: string, memberId: string) {
    return this.request(`/api/settings/${userId}/team/${memberId}`, {
      method: 'DELETE',
    });
  }

  // Billing (Mock endpoints for now)
  static async getBillingInfo(userId: string) {
    return {
      success: true,
      data: {
        plan: 'Pro',
        amount: 29.99,
        billing_cycle: 'monthly',
        next_billing: '2024-10-01',
        usage: {
          repositories: { used: 12, limit: 50 },
          ai_requests: { used: 2773, limit: 10000 },
          team_members: { used: 3, limit: 10 },
          storage: { used: 2.1, limit: 100 }
        }
      }
    };
  }

  static async getInvoices(userId: string) {
    return {
      success: true,
      data: [
        {
          id: 'INV-2024-001',
          date: '2024-09-01',
          amount: '$29.99',
          status: 'paid',
          download_url: '#'
        }
      ]
    };
  }

  // Notifications
  static async updateNotificationSettings(userId: string, settings: any) {
    return this.updateSettings(userId, { notifications: settings });
  }

  // Security
  static async updateSecuritySettings(userId: string, settings: any) {
    return this.updateSettings(userId, { security: settings });
  }

  static async getActiveSessions(userId: string) {
    return {
      success: true,
      data: [
        {
          id: 1,
          device: 'MacBook Pro',
          location: 'San Francisco, CA',
          ip: '192.168.1.100',
          last_active: '2 minutes ago',
          current: true
        }
      ]
    };
  }

  static async revokeSession(userId: string, sessionId: string) {
    return {
      success: true,
      data: null
    };
  }
}