import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
export interface Webhook {
  id: string;
  name: string;
  url: string;
  events: string[];
  status: string;
  created_at: string;
  last_delivery?: string;
}

export interface CreateWebhookRequest {
  name: string;
  url: string;
  events: string[];
}

export function useWebhooks(userId: string = 'default') {
  const [webhooks, setWebhooks] = useState<Webhook[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchWebhooks = async () => {
    try {
      setLoading(true);
      const result = await apiClient.get(`/api/settings/${userId}/webhooks`) as any;

      if (result?.success) {
        setWebhooks(result.data || []);
        setError(null);
      } else {
        setError(result?.error || 'Failed to fetch webhooks');
      }
    } catch (err) {
      setError('Network error while fetching webhooks');
    } finally {
      setLoading(false);
    }
  };

  const createWebhook = async (webhookData: CreateWebhookRequest) => {
    try {
      const result = await apiClient.post(`/api/settings/${userId}/webhooks`, webhookData) as any;

      if (result?.success) {
        await fetchWebhooks(); 
        return result.data;
      } else {
        setError(result?.error || 'Failed to create webhook');
        return null;
      }
    } catch (err) {
      setError('Network error while creating webhook');
      return null;
    }
  };

  const deleteWebhook = async (webhookId: string) => {
    try {
      const result = await apiClient.delete(`/api/settings/${userId}/webhooks/${webhookId}`) as any;

      if (result?.success) {
        await fetchWebhooks(); 
        return true;
      } else {
        setError(result?.error || 'Failed to delete webhook');
        return false;
      }
    } catch (err) {
      setError('Network error while deleting webhook');
      return false;
    }
  };

  useEffect(() => {
    fetchWebhooks();
  }, [userId]);

  return {
    webhooks,
    loading,
    error,
    createWebhook,
    deleteWebhook,
    refetch: fetchWebhooks,
  };
}