/* eslint-disable @typescript-eslint/no-unused-vars */
import { apiClient } from './api';

export class SettingsAPI {
  static async getSettings(userId: string) {
    return apiClient.get(`/api/settings/${userId}`);
  }

  static async updateSettings(userId: string, settings: unknown) {
    return apiClient.put(`/api/settings/${userId}`, settings);
  }

  static async getApiTokens(userId: string) {
    return apiClient.get(`/api/settings/${userId}/api-tokens`);
  }

  static async createApiToken(userId: string, tokenData: { name: string; permissions: string[] }) {
    return apiClient.post(`/api/settings/${userId}/api-tokens`, tokenData);
  }

  static async deleteApiToken(userId: string, tokenId: string) {
    return apiClient.delete(`/api/settings/${userId}/api-tokens/${tokenId}`);
  }

  static async getWebhooks(userId: string) {
    return apiClient.get(`/api/settings/${userId}/webhooks`);
  }

  static async createWebhook(userId: string, webhookData: { name: string; url: string; events: string[] }) {
    return apiClient.post(`/api/settings/${userId}/webhooks`, webhookData);
  }

  static async deleteWebhook(userId: string, webhookId: string) {
    return apiClient.delete(`/api/settings/${userId}/webhooks/${webhookId}`);
  }

  static async getTeamMembers(userId: string) {
    return apiClient.get(`/api/settings/${userId}/team`);
  }

  static async inviteTeamMember(userId: string, memberData: { email: string; role: string }) {
    return apiClient.post(`/api/settings/${userId}/team`, memberData);
  }

  static async removeTeamMember(userId: string, memberId: string) {
    return apiClient.delete(`/api/settings/${userId}/team/${memberId}`);
  }

  // Local/mocked responses for billing/invoices/sessions
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

  static async updateNotificationSettings(userId: string, settings: unknown) {
    return this.updateSettings(userId, { notifications: settings });
  }

  static async updateSecuritySettings(userId: string, settings: unknown) {
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