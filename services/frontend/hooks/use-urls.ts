import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';

export interface UrlRecord {
  id: string;
  url: string;
  title: string;
  description?: string;
  tags: string[];
  created_at: string;
  status: string;
}

export interface CreateUrlData {
  url: string;
  title?: string;
  description?: string;
  tags?: string[];
}

export function useUrls() {
  const [urls, setUrls] = useState<UrlRecord[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchUrls = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const result = await apiClient.getUrls();
      if (result.success) {
        setUrls(result.data || []);
      } else {
        setError(result.message || 'Failed to fetch URLs');
      }
    } catch (err) {
      setError('Failed to fetch URLs');
    } finally {
      setIsLoading(false);
    }
  };

  const createUrl = async (data: CreateUrlData) => {
    try {
      const result = await apiClient.createUrl(data);
      if (result.success && result.data) {
        setUrls(prev => [...prev, result.data]);
        return { success: true, data: result.data };
      } else {
        return { success: false, error: result.message || 'Failed to create URL' };
      }
    } catch (err) {
      return { success: false, error: 'Failed to create URL' };
    }
  };

  const deleteUrl = async (id: string) => {
    try {
      const result = await apiClient.deleteUrl(id);
      if (result.success) {
        setUrls(prev => prev.filter(url => url.id !== id));
        return { success: true };
      } else {
        return { success: false, error: result.message || 'Failed to delete URL' };
      }
    } catch (err) {
      return { success: false, error: 'Failed to delete URL' };
    }
  };

  useEffect(() => {
    fetchUrls();
  }, []);

  return {
    urls,
    isLoading,
    error,
    fetchUrls,
    createUrl,
    deleteUrl,
    refetch: fetchUrls,
  };
}