import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';

export interface ProfileSettings {
  first_name: string;
  last_name: string;
  email: string;
  bio?: string;
  location?: string;
  website?: string;
  social_links: Record<string, string>;
}

export interface NotificationSettings {
  email_notifications: boolean;
  push_notifications: boolean;
  security_alerts: boolean;
}

export interface SecuritySettings {
  two_factor_enabled: boolean;
  session_timeout: number;
}

export interface UserSettings {
  user_id: string;
  profile: ProfileSettings;
  notifications: NotificationSettings;
  security: SecuritySettings;
}

export interface UpdateSettingsRequest {
  profile?: ProfileSettings;
  notifications?: NotificationSettings;
  security?: SecuritySettings;
}

export function useSettings(userId: string = 'default') {
  const [settings, setSettings] = useState<UserSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchSettings = async () => {
    try {
      setLoading(true);
      const result = await apiClient.get(`/api/settings/${userId}`) as any;

      if (result?.success) {
        setSettings(result.data);
        setError(null);
      } else {
        setError(result?.error || 'Failed to fetch settings');
      }
    } catch (err) {
      setError('Network error while fetching settings');
    } finally {
      setLoading(false);
    }
  };

  const updateSettings = async (updates: UpdateSettingsRequest) => {
    try {
      setLoading(true);
      const result = await apiClient.put(`/api/settings/${userId}`, updates) as any;

      if (result?.success) {
        setSettings(result.data);
        setError(null);
        return true;
      } else {
        setError(result?.error || 'Failed to update settings');
        return false;
      }
    } catch (err) {
      setError('Network error while updating settings');
      return false;
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, [userId]);

  return {
    settings,
    loading,
    error,
    updateSettings,
    refetch: fetchSettings,
  };
}