import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
export interface ApiToken {
  id: string;
  name: string;
  token: string;
  permissions: string[];
  created_at: string;
  last_used?: string;
}

export interface CreateApiTokenRequest {
  name: string;
  permissions: string[];
}

export function useApiTokens(userId: string = 'default') {
  const [tokens, setTokens] = useState<ApiToken[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTokens = async () => {
    try {
      setLoading(true);
      const result = await apiClient.get(`/api/settings/${userId}/api-tokens`) as any;

      if (result?.success) {
        setTokens(result.data || []);
        setError(null);
      } else {
        setError(result?.error || 'Failed to fetch API tokens');
      }
    } catch (err) {
      setError('Network error while fetching API tokens');
    } finally {
      setLoading(false);
    }
  };

  const createToken = async (tokenData: CreateApiTokenRequest) => {
    try {
      const result = await apiClient.post(`/api/settings/${userId}/api-tokens`, tokenData) as any;

      if (result?.success) {
        await fetchTokens(); 
        return result.data;
      } else {
        setError(result?.error || 'Failed to create API token');
        return null;
      }
    } catch (err) {
      setError('Network error while creating API token');
      return null;
    }
  };

  const deleteToken = async (tokenId: string) => {
    try {
      const result = await apiClient.delete(`/api/settings/${userId}/api-tokens/${tokenId}`) as any;

      if (result?.success) {
        await fetchTokens(); 
        return true;
      } else {
        setError(result?.error || 'Failed to delete API token');
        return false;
      }
    } catch (err) {
      setError('Network error while deleting API token');
      return false;
    }
  };

  useEffect(() => {
    fetchTokens();
  }, [userId]);

  return {
    tokens,
    loading,
    error,
    createToken,
    deleteToken,
    refetch: fetchTokens,
  };
}