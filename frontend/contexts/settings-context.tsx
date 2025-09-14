"use client";

import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { SettingsAPI } from '@/lib/settings-api';
import { UserSettings } from '@/hooks/use-settings';

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
        SettingsAPI.getSettings(userId),
        SettingsAPI.getApiTokens(userId),
        SettingsAPI.getWebhooks(userId),
        SettingsAPI.getTeamMembers(userId),
      ]);

      if (settingsResult.success) {
        dispatch({ type: 'SET_SETTINGS', payload: settingsResult.data });
      }
      
      if (tokensResult.success) {
        dispatch({ type: 'SET_API_TOKENS', payload: tokensResult.data || [] });
      }
      
      if (webhooksResult.success) {
        dispatch({ type: 'SET_WEBHOOKS', payload: webhooksResult.data || [] });
      }
      
      if (teamResult.success) {
        dispatch({ type: 'SET_TEAM_MEMBERS', payload: teamResult.data || [] });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Failed to load settings data' });
    }
  };

  const updateSettings = async (updates: any) => {
    const result = await SettingsAPI.updateSettings(userId, updates);
    if (result.success) {
      dispatch({ type: 'UPDATE_SETTINGS', payload: updates });
      return true;
    } else {
      dispatch({ type: 'SET_ERROR', payload: result.error || 'Failed to update settings' });
      return false;
    }
  };

  const createApiToken = async (tokenData: any) => {
    const result = await SettingsAPI.createApiToken(userId, tokenData);
    if (result.success) {
      const tokensResult = await SettingsAPI.getApiTokens(userId);
      if (tokensResult.success) {
        dispatch({ type: 'SET_API_TOKENS', payload: tokensResult.data || [] });
      }
      return result.data;
    }
    return null;
  };

  const deleteApiToken = async (tokenId: string) => {
    const result = await SettingsAPI.deleteApiToken(userId, tokenId);
    if (result.success) {
      const tokensResult = await SettingsAPI.getApiTokens(userId);
      if (tokensResult.success) {
        dispatch({ type: 'SET_API_TOKENS', payload: tokensResult.data || [] });
      }
      return true;
    }
    return false;
  };

  const createWebhook = async (webhookData: any) => {
    const result = await SettingsAPI.createWebhook(userId, webhookData);
    if (result.success) {
      const webhooksResult = await SettingsAPI.getWebhooks(userId);
      if (webhooksResult.success) {
        dispatch({ type: 'SET_WEBHOOKS', payload: webhooksResult.data || [] });
      }
      return result.data;
    }
    return null;
  };

  const deleteWebhook = async (webhookId: string) => {
    const result = await SettingsAPI.deleteWebhook(userId, webhookId);
    if (result.success) {
      const webhooksResult = await SettingsAPI.getWebhooks(userId);
      if (webhooksResult.success) {
        dispatch({ type: 'SET_WEBHOOKS', payload: webhooksResult.data || [] });
      }
      return true;
    }
    return false;
  };

  const inviteTeamMember = async (memberData: any) => {
    const result = await SettingsAPI.inviteTeamMember(userId, memberData);
    if (result.success) {
      const teamResult = await SettingsAPI.getTeamMembers(userId);
      if (teamResult.success) {
        dispatch({ type: 'SET_TEAM_MEMBERS', payload: teamResult.data || [] });
      }
      return result.data;
    }
    return null;
  };

  const removeTeamMember = async (memberId: string) => {
    const result = await SettingsAPI.removeTeamMember(userId, memberId);
    if (result.success) {
      const teamResult = await SettingsAPI.getTeamMembers(userId);
      if (teamResult.success) {
        dispatch({ type: 'SET_TEAM_MEMBERS', payload: teamResult.data || [] });
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