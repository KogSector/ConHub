"use client";

import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { SettingsAPI } from '@/lib/settings-api';
import { UserSettings } from '@/hooks/use-settings';
import { ApiResponse } from '@/lib/api';

interface SettingsState {
  settings: UserSettings | null;
  loading: boolean;
  error: string | null;
  apiTokens: any[];
  webhooks: any[];
  teamMembers: any[];
}

type SettingsAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_SETTINGS'; payload: UserSettings }
  | { type: 'SET_API_TOKENS'; payload: any[] }
  | { type: 'SET_WEBHOOKS'; payload: any[] }
  | { type: 'SET_TEAM_MEMBERS'; payload: any[] }
  | { type: 'UPDATE_SETTINGS'; payload: Partial<UserSettings> };

const initialState: SettingsState = {
  settings: null,
  loading: true,
  error: null,
  apiTokens: [],
  webhooks: [],
  teamMembers: [],
};

function settingsReducer(state: SettingsState, action: SettingsAction): SettingsState {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
    case 'SET_SETTINGS':
      return { ...state, settings: action.payload, loading: false, error: null };
    case 'SET_API_TOKENS':
      return { ...state, apiTokens: action.payload };
    case 'SET_WEBHOOKS':
      return { ...state, webhooks: action.payload };
    case 'SET_TEAM_MEMBERS':
      return { ...state, teamMembers: action.payload };
    case 'UPDATE_SETTINGS':
      return {
        ...state,
        settings: state.settings ? { ...state.settings, ...action.payload } : null,
      };
    default:
      return state;
  }
}

interface SettingsContextType extends SettingsState {
  updateSettings: (updates: any) => Promise<boolean>;
  createApiToken: (tokenData: any) => Promise<any>;
  deleteApiToken: (tokenId: string) => Promise<boolean>;
  createWebhook: (webhookData: any) => Promise<any>;
  deleteWebhook: (webhookId: string) => Promise<boolean>;
  inviteTeamMember: (memberData: any) => Promise<any>;
  removeTeamMember: (memberId: string) => Promise<boolean>;
  refreshData: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export function SettingsProvider({ children, userId = 'default' }: { children: React.ReactNode; userId?: string }) {
  const [state, dispatch] = useReducer(settingsReducer, initialState);

  const loadAllData = async () => {
    dispatch({ type: 'SET_LOADING', payload: true });
    
    try {
      const [settingsResult, tokensResult, webhooksResult, teamResult] = await Promise.all([
        SettingsAPI.getSettings(userId) as Promise<ApiResponse>,
        SettingsAPI.getApiTokens(userId) as Promise<ApiResponse>,
        SettingsAPI.getWebhooks(userId) as Promise<ApiResponse>,
        SettingsAPI.getTeamMembers(userId) as Promise<ApiResponse>,
      ]);

      if (settingsResult.success && settingsResult.data) {
        dispatch({ type: 'SET_SETTINGS', payload: settingsResult.data as UserSettings });
      }
      
      if (tokensResult.success && tokensResult.data) {
        dispatch({ type: 'SET_API_TOKENS', payload: (tokensResult.data as unknown[]) || [] });
      }
      
      if (webhooksResult.success && webhooksResult.data) {
        dispatch({ type: 'SET_WEBHOOKS', payload: (webhooksResult.data as unknown[]) || [] });
      }
      
      if (teamResult.success && teamResult.data) {
        dispatch({ type: 'SET_TEAM_MEMBERS', payload: (teamResult.data as unknown[]) || [] });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Failed to load settings data' });
    }
  };

  const updateSettings = async (updates: Record<string, unknown>) => {
    const result = await SettingsAPI.updateSettings(userId, updates) as ApiResponse;
    if (result.success) {
      dispatch({ type: 'UPDATE_SETTINGS', payload: updates });
      return true;
    } else {
      dispatch({ type: 'SET_ERROR', payload: result.error || 'Failed to update settings' });
      return false;
    }
  };

  const createApiToken = async (tokenData: { name: string; permissions: string[] }) => {
    const result = await SettingsAPI.createApiToken(userId, tokenData) as ApiResponse;
    if (result.success) {
      const tokensResult = await SettingsAPI.getApiTokens(userId) as ApiResponse;
      if (tokensResult.success && tokensResult.data) {
        dispatch({ type: 'SET_API_TOKENS', payload: (tokensResult.data as unknown[]) || [] });
      }
      return result.data;
    }
    return null;
  };

  const deleteApiToken = async (tokenId: string) => {
    const result = await SettingsAPI.deleteApiToken(userId, tokenId) as ApiResponse;
    if (result.success) {
      const tokensResult = await SettingsAPI.getApiTokens(userId) as ApiResponse;
      if (tokensResult.success && tokensResult.data) {
        dispatch({ type: 'SET_API_TOKENS', payload: (tokensResult.data as unknown[]) || [] });
      }
      return true;
    }
    return false;
  };

  const createWebhook = async (webhookData: { name: string; url: string; events: string[] }) => {
    const result = await SettingsAPI.createWebhook(userId, webhookData) as ApiResponse;
    if (result.success) {
      const webhooksResult = await SettingsAPI.getWebhooks(userId) as ApiResponse;
      if (webhooksResult.success && webhooksResult.data) {
        dispatch({ type: 'SET_WEBHOOKS', payload: (webhooksResult.data as unknown[]) || [] });
      }
      return result.data;
    }
    return null;
  };

  const deleteWebhook = async (webhookId: string) => {
    const result = await SettingsAPI.deleteWebhook(userId, webhookId) as ApiResponse;
    if (result.success) {
      const webhooksResult = await SettingsAPI.getWebhooks(userId) as ApiResponse;
      if (webhooksResult.success && webhooksResult.data) {
        dispatch({ type: 'SET_WEBHOOKS', payload: (webhooksResult.data as unknown[]) || [] });
      }
      return true;
    }
    return false;
  };

  const inviteTeamMember = async (memberData: { email: string; role: string }) => {
    const result = await SettingsAPI.inviteTeamMember(userId, memberData) as ApiResponse;
    if (result.success) {
      const teamResult = await SettingsAPI.getTeamMembers(userId) as ApiResponse;
      if (teamResult.success && teamResult.data) {
        dispatch({ type: 'SET_TEAM_MEMBERS', payload: (teamResult.data as unknown[]) || [] });
      }
      return result.data;
    }
    return null;
  };

  const removeTeamMember = async (memberId: string) => {
    const result = await SettingsAPI.removeTeamMember(userId, memberId) as ApiResponse;
    if (result.success) {
      const teamResult = await SettingsAPI.getTeamMembers(userId) as ApiResponse;
      if (teamResult.success && teamResult.data) {
        dispatch({ type: 'SET_TEAM_MEMBERS', payload: (teamResult.data as unknown[]) || [] });
      }
      return true;
    }
    return false;
  };

  useEffect(() => {
    loadAllData();
  }, [userId]);

  const contextValue: SettingsContextType = {
    ...state,
    updateSettings,
    createApiToken,
    deleteApiToken,
    createWebhook,
    deleteWebhook,
    inviteTeamMember,
    removeTeamMember,
    refreshData: loadAllData,
  };

  return (
    <SettingsContext.Provider value={contextValue}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettingsContext() {
  const context = useContext(SettingsContext);
  if (context === undefined) {
    throw new Error('useSettingsContext must be used within a SettingsProvider');
  }
  return context;
}