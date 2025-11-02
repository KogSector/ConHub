"use client";

import React, { createContext, useContext, useReducer, useEffect, useCallback } from 'react';
import { SettingsAPI } from '@/lib/settings-api';
import { UserSettings } from '@/hooks/use-settings';
import { ApiResponse } from '@/lib/api';
import { ApiToken, Webhook, TeamMember, CreateApiTokenData, CreateWebhookData, InviteTeamMemberData } from '@/types/settings';

interface SettingsState {
  settings: UserSettings | null;
  loading: boolean;
  error: string | null;
  apiTokens: ApiToken[];
  webhooks: Webhook[];
  teamMembers: TeamMember[];
}

type SettingsAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_SETTINGS'; payload: UserSettings }
  | { type: 'SET_API_TOKENS'; payload: ApiToken[] }
  | { type: 'SET_WEBHOOKS'; payload: Webhook[] }
  | { type: 'SET_TEAM_MEMBERS'; payload: TeamMember[] }
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
  updateSettings: (updates: Record<string, unknown>) => Promise<boolean>;
  createApiToken: (tokenData: CreateApiTokenData) => Promise<ApiToken | null>;
  deleteApiToken: (tokenId: string) => Promise<boolean>;
  createWebhook: (webhookData: CreateWebhookData) => Promise<Webhook | null>;
  deleteWebhook: (webhookId: string) => Promise<boolean>;
  inviteTeamMember: (memberData: InviteTeamMemberData) => Promise<TeamMember | null>;
  removeTeamMember: (memberId: string) => Promise<boolean>;
  refreshData: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export function SettingsProvider({ children, userId = 'default' }: { children: React.ReactNode; userId?: string }) {
  const [state, dispatch] = useReducer(settingsReducer, initialState);

  // Helper function to handle API result and dispatch
  const handleApiResult = <T>(result: ApiResponse, actionType: SettingsAction['type'], fallback: T[] = []) => {
    if (result.success && result.data) {
      dispatch({ type: actionType, payload: (result.data as T[]) || fallback });
    }
  };

  // Helper function to refresh specific data type
  const refreshDataType = async <T>(
    apiCall: () => Promise<ApiResponse>,
    actionType: SettingsAction['type'],
    fallback: T[] = []
  ) => {
    try {
      const result = await apiCall();
      handleApiResult<T>(result, actionType, fallback);
    } catch (error) {
      console.error(`Failed to refresh ${actionType}:`, error);
    }
  };

  const loadAllData = useCallback(async () => {
    dispatch({ type: 'SET_LOADING', payload: true });
    
    try {
      const [settingsResult, tokensResult, webhooksResult, teamResult] = await Promise.all([
        SettingsAPI.getSettings(userId) as Promise<ApiResponse>,
        SettingsAPI.getApiTokens(userId) as Promise<ApiResponse>,
        SettingsAPI.getWebhooks(userId) as Promise<ApiResponse>,
        SettingsAPI.getTeamMembers(userId) as Promise<ApiResponse>,
      ]);

      // Handle settings result
      if (settingsResult.success && settingsResult.data) {
        dispatch({ type: 'SET_SETTINGS', payload: settingsResult.data as UserSettings });
      }
      
      // Handle other results using helper
      handleApiResult<ApiToken>(tokensResult, 'SET_API_TOKENS');
      handleApiResult<Webhook>(webhooksResult, 'SET_WEBHOOKS');
      handleApiResult<TeamMember>(teamResult, 'SET_TEAM_MEMBERS');
      
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Failed to load settings data' });
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, [userId]);

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

  const createApiToken = async (tokenData: CreateApiTokenData): Promise<ApiToken | null> => {
    const result = await SettingsAPI.createApiToken(userId, tokenData) as ApiResponse;
    if (result.success) {
      await refreshDataType<ApiToken>(() => SettingsAPI.getApiTokens(userId) as Promise<ApiResponse>, 'SET_API_TOKENS');
      return result.data as ApiToken;
    }
    return null;
  };

  const deleteApiToken = async (tokenId: string): Promise<boolean> => {
    const result = await SettingsAPI.deleteApiToken(userId, tokenId) as ApiResponse;
    if (result.success) {
      await refreshDataType<ApiToken>(() => SettingsAPI.getApiTokens(userId) as Promise<ApiResponse>, 'SET_API_TOKENS');
      return true;
    }
    return false;
  };

  const createWebhook = async (webhookData: CreateWebhookData): Promise<Webhook | null> => {
    const result = await SettingsAPI.createWebhook(userId, webhookData) as ApiResponse;
    if (result.success) {
      await refreshDataType<Webhook>(() => SettingsAPI.getWebhooks(userId) as Promise<ApiResponse>, 'SET_WEBHOOKS');
      return result.data as Webhook;
    }
    return null;
  };

  const deleteWebhook = async (webhookId: string): Promise<boolean> => {
    const result = await SettingsAPI.deleteWebhook(userId, webhookId) as ApiResponse;
    if (result.success) {
      await refreshDataType<Webhook>(() => SettingsAPI.getWebhooks(userId) as Promise<ApiResponse>, 'SET_WEBHOOKS');
      return true;
    }
    return false;
  };

  const inviteTeamMember = async (memberData: InviteTeamMemberData): Promise<TeamMember | null> => {
    const result = await SettingsAPI.inviteTeamMember(userId, memberData) as ApiResponse;
    if (result.success) {
      await refreshDataType<TeamMember>(() => SettingsAPI.getTeamMembers(userId) as Promise<ApiResponse>, 'SET_TEAM_MEMBERS');
      return result.data as TeamMember;
    }
    return null;
  };

  const removeTeamMember = async (memberId: string): Promise<boolean> => {
    const result = await SettingsAPI.removeTeamMember(userId, memberId) as ApiResponse;
    if (result.success) {
      await refreshDataType<TeamMember>(() => SettingsAPI.getTeamMembers(userId) as Promise<ApiResponse>, 'SET_TEAM_MEMBERS');
      return true;
    }
    return false;
  };

  useEffect(() => {
    loadAllData();
  }, [loadAllData]);

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