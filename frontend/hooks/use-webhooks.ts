import { useState, useEffect } from 'react';

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

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

export function useWebhooks(userId: string = 'default') {
  const [webhooks, setWebhooks] = useState<Webhook[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchWebhooks = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${API_BASE_URL}/api/settings/${userId}/webhooks`);
      const result = await response.json();
      
      if (result.success) {
        setWebhooks(result.data || []);
        setError(null);
      } else {
        setError(result.error || 'Failed to fetch webhooks');
      }
    } catch (err) {
      setError('Network error while fetching webhooks');
    } finally {
      setLoading(false);
    }
  };

  const createWebhook = async (webhookData: CreateWebhookRequest) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/${userId}/webhooks`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(webhookData),
      });
      
      const result = await response.json();
      
      if (result.success) {
        await fetchWebhooks(); // Refresh the list
        return result.data;
      } else {
        setError(result.error || 'Failed to create webhook');
        return null;
      }
    } catch (err) {
      setError('Network error while creating webhook');
      return null;
    }
  };

  const deleteWebhook = async (webhookId: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/${userId}/webhooks/${webhookId}`, {
        method: 'DELETE',
      });
      
      const result = await response.json();
      
      if (result.success) {
        await fetchWebhooks(); // Refresh the list
        return true;
      } else {
        setError(result.error || 'Failed to delete webhook');
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