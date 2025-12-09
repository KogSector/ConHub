"use client"

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Trash2, RefreshCw, Plus } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { apiClient, securityApiClient, unwrapResponse } from '@/lib/api';
import { useAuth } from '@/hooks/use-auth';
import Link from 'next/link';
import { ArrowLeft, Share2 } from 'lucide-react';
import { ProfileAvatar } from '@/components/ui/ProfileAvatar';
import GitHubIcon from '@/components/icons/GitHubIcon'
import GitLabIcon from '@/components/icons/GitLabIcon'
import BitbucketIcon from '@/components/icons/BitbucketIcon'
import SlackIcon from '@/components/icons/SlackIcon'
import GoogleDriveIcon from '@/components/icons/GoogleDriveIcon'
import GmailIcon from '@/components/icons/GmailIcon'
import DropboxIcon from '@/components/icons/DropboxIcon'
import LinkedInIcon from '@/components/icons/LinkedInIcon'
import NotionIcon from '@/components/icons/NotionIcon'
import JiraIcon from '@/components/icons/JiraIcon'
import ConfluenceIcon from '@/components/icons/ConfluenceIcon'
import CustomAppsIcon from '@/components/icons/CustomAppsIcon'

interface SocialConnection {
  id: string;
  platform: 'slack' | 'notion' | 'google_drive' | 'gmail' | 'dropbox' | 'linkedin' | 'github' | 'bitbucket' | 'gitlab' | 'jira' | 'confluence' | 'custom_apps';
  username: string;
  is_active: boolean;
  connected_at: string;
  last_sync: string | null;
}

const PLATFORM_CONFIGS = {
  slack: { name: 'Slack', description: 'Connect to sync messages and channels', icon: (cls: string) => <SlackIcon className={cls} /> },
  notion: { name: 'Notion', description: 'Sync pages and databases', icon: (cls: string) => <NotionIcon className={cls} /> },
  google_drive: { name: 'Google Drive', description: 'Access files and documents', icon: (cls: string) => <GoogleDriveIcon className={cls} /> },
  gmail: { name: 'Gmail', description: 'Sync email conversations', icon: (cls: string) => <GmailIcon className={cls} /> },
  dropbox: { name: 'Dropbox', description: 'Sync files and folders', icon: (cls: string) => <DropboxIcon className={cls} /> },
  linkedin: { name: 'LinkedIn', description: 'Connect professional network', icon: (cls: string) => <LinkedInIcon className={cls} /> },
  github: { name: 'GitHub', description: 'Connect your GitHub account', icon: (cls: string) => <GitHubIcon className={cls} /> },
  bitbucket: { name: 'Bitbucket', description: 'Connect your Bitbucket account', icon: (cls: string) => <BitbucketIcon className={cls} /> },
  gitlab: { name: 'GitLab', description: 'Connect your GitLab account', icon: (cls: string) => <GitLabIcon className={cls} /> },
  jira: { name: 'Jira', description: 'Sync issues and projects', icon: (cls: string) => <JiraIcon className={cls} /> },
  confluence: { name: 'Confluence', description: 'Sync pages and spaces', icon: (cls: string) => <ConfluenceIcon className={cls} /> },
  custom_apps: { name: 'Custom Apps', description: 'Integrate third party apps', icon: (cls: string) => <CustomAppsIcon className={cls} /> },
} as const;

export function SocialConnections() {
  const [connections, setConnections] = useState<SocialConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState<string | null>(null);
  // Use a Set in a ref to track codes that have been processed (prevents duplicate exchanges)
  const processedCodesRef = useRef<Set<string>>(new Set());
  const { toast } = useToast();
  const { token, refreshConnections } = useAuth();

  const fetchConnections = useCallback(async () => {
    // Do not hit the services until we have an Auth0 access token;
    // otherwise we get an automatic 401 and show a spurious error toast.
    if (!token) {
      setLoading(false);
      return;
    }

    setLoading(true);
    try {
      const headers: Record<string, string> = { Authorization: `Bearer ${token}` };
      
      // Fetch from both auth service (GitHub/Bitbucket/GitLab OAuth) and security service
      const [authResp, securityResp] = await Promise.allSettled([
        apiClient.get('/api/auth/connections', headers),
        securityApiClient.get('/api/security/connections', headers)
      ]);
      
      // Merge connections from both sources, avoiding duplicates by platform
      const authData = authResp.status === 'fulfilled' 
        ? (unwrapResponse<SocialConnection[]>(authResp.value) ?? [])
        : [];
      const securityData = securityResp.status === 'fulfilled'
        ? (unwrapResponse<SocialConnection[]>(securityResp.value) ?? [])
        : [];
      
      // Use a Map to dedupe by platform (auth service takes precedence for OAuth providers)
      const connectionMap = new Map<string, SocialConnection>();
      securityData.forEach(conn => connectionMap.set(conn.platform, conn));
      authData.forEach(conn => connectionMap.set(conn.platform, conn)); // Auth overwrites security

      const merged = Array.from(connectionMap.values());
      console.log('[SocialConnections] fetched connections', {
        tokenPresent: !!token,
        authData,
        securityData,
        merged,
      });

      setConnections(merged);
    } catch (error) {
      console.error('Error fetching connections:', error);
      toast({
        title: "Error",
        description: "Failed to fetch social connections",
        variant: "destructive"
      });
    } finally {
      setLoading(false);
    }
  }, [toast, token]);

  useEffect(() => {
    fetchConnections();
    const handler = async (e: MessageEvent) => {
      const dataUnknown: unknown = e.data
      if (typeof dataUnknown !== 'object' || dataUnknown === null || !('type' in dataUnknown)) {
        return
      }
      
      const data = dataUnknown as { type: string; provider?: string; code?: string; error?: string }
      
      if (data.type === 'oauth-connected') {
        // Legacy: popup already did the exchange
        fetchConnections()
      } else if (data.type === 'oauth-code' && data.provider && data.code) {
        // Prevent duplicate exchange calls - OAuth codes are single-use
        const codeKey = `${data.provider}:${data.code}`
        if (processedCodesRef.current.has(codeKey)) {
          console.log('OAuth code already processed, skipping duplicate:', codeKey)
          return
        }
        processedCodesRef.current.add(codeKey)
        
        // New: popup sent us the code, we do the exchange here (we have the token)
        try {
          const storedToken = localStorage.getItem('auth_token')
          if (!storedToken) {
            toast({
              title: "Error",
              description: "Please log in first before connecting accounts",
              variant: "destructive"
            })
            return
          }
          const headers: Record<string, string> = { Authorization: `Bearer ${storedToken}` }
          await apiClient.post('/api/auth/oauth/exchange', { provider: data.provider, code: data.code }, headers)
          toast({
            title: "Success",
            description: `${data.provider} connected successfully!`,
          })
          fetchConnections()
          // Also refresh the global connections in useAuth hook so other components see the update
          if (refreshConnections) refreshConnections()
        } catch (error: any) {
          console.error('OAuth exchange error:', error)
          const errorMsg = error?.message || `Failed to connect ${data.provider}`
          
          // Provide specific guidance for common OAuth errors
          if (errorMsg.includes('bad_verification_code') || errorMsg.includes('incorrect or expired')) {
            toast({
              title: "Authorization Expired",
              description: "The GitHub authorization code has expired. Please click Connect again to restart.",
              variant: "destructive"
            })
          } else if (errorMsg.includes('redirect_uri_mismatch')) {
            toast({
              title: "Configuration Error",
              description: "OAuth callback URL mismatch. Please check the GitHub OAuth app settings.",
              variant: "destructive"
            })
          } else {
            toast({
              title: "Error",
              description: errorMsg,
              variant: "destructive"
            })
          }
        }
      } else if (data.type === 'oauth-error' && data.error) {
        toast({
          title: "Error",
          description: data.error,
          variant: "destructive"
        })
      }
    }
    window.addEventListener('message', handler)
    return () => window.removeEventListener('message', handler)
  }, [fetchConnections, toast]);

  

  const connectPlatform = async (platform: string) => {
    try {
      // OAuth providers handled through auth service
      const oauthProviders = ['github', 'bitbucket', 'gitlab', 'google', 'google_drive'];
      const providerForAuth = platform === 'google_drive' ? 'google' : platform;
      
      if (oauthProviders.includes(platform)) {
        const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
        const resp = await apiClient.get<{ url: string; state: string }>(`/api/auth/oauth/url?provider=${providerForAuth}`, headers)
        const { url: authUrl } = resp
        if (authUrl) {
          // Open popup - the callback page will send the code via postMessage
          // and we'll handle the exchange in the message handler above
          window.open(authUrl, '_blank', 'width=500,height=700')
        } else {
          toast({ title: 'Error', description: `Failed to get ${platform} auth URL. Please check OAuth configuration.`, variant: 'destructive' })
        }
        return
      }
      
      // Other platforms via security service
      const resp = await securityApiClient.post('/api/security/connections/connect', { platform });
      const payload = unwrapResponse<{ credentials?: { auth_url?: string }; account?: { credentials?: { auth_url?: string } } }>(resp) ?? {}
      const authUrl = payload?.credentials?.auth_url || payload?.account?.credentials?.auth_url
      if (authUrl) {
        window.open(authUrl, '_blank', 'width=500,height=600')
        setTimeout(() => {
          fetchConnections();
        }, 3000);
      } else {
        toast({ title: 'Error', description: `Failed to connect to ${platform}`, variant: 'destructive' });
      }
    } catch (error) {
      console.error('Error connecting platform:', error);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      toast({ title: 'Error', description: `Failed to connect to ${platform}: ${errorMessage}`, variant: 'destructive' });
    }
  };

  const disconnectPlatform = async (connectionId: string, platform: string) => {
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      
      // Route to the correct service based on platform type
      // VCS providers (github, bitbucket, gitlab) are managed by auth service
      // Other platforms (gmail, google_drive, dropbox, slack, etc.) are managed by security service
      const vcsProviders = ['github', 'bitbucket', 'gitlab'];
      const isVcsProvider = vcsProviders.includes(platform);
      
      if (isVcsProvider) {
        // Only call auth service for VCS providers
        await apiClient.delete(`/api/auth/connections/${connectionId}`, headers);
      } else {
        // Only call security service for other platforms
        await securityApiClient.delete(`/api/security/connections/${connectionId}`, headers);
      }
      
      setConnections(prev => prev.filter(conn => conn.id !== connectionId));
      toast({
        title: "Success",
        description: "Platform disconnected successfully"
      });
    } catch (error) {
      console.error('Error disconnecting platform:', error);
      toast({
        title: "Error",
        description: "Failed to disconnect platform",
        variant: "destructive"
      });
    }
  };

  const syncPlatform = async (connectionId: string, platform: string) => {
    setSyncing(connectionId);
    try {
      await securityApiClient.post(`/api/security/connections/oauth/callback`, { platform, code: 'dummy' });
      toast({
        title: "Success",
        description: `${PLATFORM_CONFIGS[platform as keyof typeof PLATFORM_CONFIGS].name} synced successfully`
      });
      fetchConnections();
    } catch (error) {
      console.error('Error syncing platform:', error);
      toast({
        title: "Error",
        description: `Failed to sync ${platform}`,
        variant: "destructive"
      });
    } finally {
      setSyncing(null);
    }
  };

  if (loading) {
    return (
      <div className="space-y-4">
        <h2 className="text-2xl font-bold">Connections</h2>
        <div className="animate-pulse space-y-4">
          {[1, 2, 3].map(i => (
            <Card key={i}>
              <CardContent className="p-6">
                <div className="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2"></div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard">
                <Button variant="ghost" size="sm">
                  <ArrowLeft className="w-4 h-4" />
                </Button>
              </Link>
              <div className="flex items-center space-x-3">
                <Share2 className="w-6 h-6 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">Connections</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>
      <div>
        <p className="text-muted-foreground">
          Connect your accounts to enhance context and collaboration across platforms
        </p>
      </div>

      {/* All Platforms Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {(Object.keys(PLATFORM_CONFIGS) as Array<keyof typeof PLATFORM_CONFIGS>).map((platform) => {
          const config = PLATFORM_CONFIGS[platform];
          const connection = connections.find(c => c.platform === platform);
          const isConnected = connection?.is_active;
          
          return (
            <Card key={platform} className="hover:shadow-md transition-shadow">
              <CardHeader className="pb-3">
                <div className="flex items-center space-x-3">
                  <div className="w-8 h-8 rounded-lg flex items-center justify-center">
                    {config.icon('w-8 h-8')}
                  </div>
                  <CardTitle className="text-lg">{config.name}</CardTitle>
                </div>
                <CardDescription>
                  {isConnected ? (
                    <span className="text-green-600 dark:text-green-400">
                      Connected
                    </span>
                  ) : (
                    config.description
                  )}
                </CardDescription>
              </CardHeader>
              <CardContent className="pt-0">
                {isConnected ? (
                  <Button
                    title={`Disconnect ${config.name}`}
                    aria-label={`Disconnect ${config.name}`}
                    onClick={() => connection && disconnectPlatform(connection.id, platform)}
                    className="w-full bg-red-600 hover:bg-red-700 text-white"
                    variant="destructive"
                  >
                    <Trash2 className="h-4 w-4 mr-2" />
                    Disconnect
                  </Button>
                ) : (
                  <Button
                    title={`Connect ${config.name}`}
                    aria-label={`Connect ${config.name}`}
                    onClick={() => connectPlatform(platform)}
                    className="w-full"
                    variant="outline"
                  >
                    <Plus className="h-4 w-4 mr-2" />
                    Connect
                  </Button>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}

export default SocialConnections;
