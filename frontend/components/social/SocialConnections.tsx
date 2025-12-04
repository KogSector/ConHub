"use client"

import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Trash2, RefreshCw, Plus } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { apiClient, securityApiClient, unwrapResponse } from '@/lib/api';
import { useAuth } from '@/hooks/use-auth';
import Link from 'next/link';
import { ArrowLeft, Share2 } from 'lucide-react';
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
  const { toast } = useToast();
  const { token } = useAuth();

  const fetchConnections = useCallback(async () => {
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      // Use the security service connections API; auth service may be disabled and
      // it does not expose /api/auth/connections.
      const resp = await securityApiClient.get('/api/security/connections', headers);
      const data = unwrapResponse<SocialConnection[]>(resp) ?? [];
      setConnections(data);
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
    const handler = (e: MessageEvent) => {
      const dataUnknown: unknown = e.data
      if (
        typeof dataUnknown === 'object' &&
        dataUnknown !== null &&
        'type' in dataUnknown &&
        (dataUnknown as { type: string }).type === 'oauth-connected'
      ) {
        fetchConnections()
      }
    }
    window.addEventListener('message', handler)
    return () => window.removeEventListener('message', handler)
  }, [fetchConnections]);

  

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
          window.open(authUrl, '_blank', 'width=500,height=700')
          // Listen for OAuth completion and refresh connections
          const checkInterval = setInterval(() => {
            fetchConnections();
          }, 3000);
          // Clear interval after 60 seconds
          setTimeout(() => clearInterval(checkInterval), 60000);
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

  const disconnectPlatform = async (connectionId: string) => {
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      // Disconnect via security service; this matches the backend routes in
      // security/src/handlers/connections.rs.
      await securityApiClient.delete(`/api/security/connections/${connectionId}`, headers);
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

  const connectedPlatforms = new Set<keyof typeof PLATFORM_CONFIGS>(
    connections.map(conn => conn.platform as keyof typeof PLATFORM_CONFIGS)
  );
  const availablePlatforms = (Object.keys(PLATFORM_CONFIGS) as Array<keyof typeof PLATFORM_CONFIGS>).filter(
    (platform) => !connectedPlatforms.has(platform)
  );

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
              <div className="h-6 w-px bg-border" />
              <Share2 className="w-5 h-5 text-primary" />
              <h2 className="text-2xl font-bold">Connections</h2>
            </div>
          </div>
        </div>
      </div>
      <div>
        <p className="text-muted-foreground">
          Connect your accounts to enhance context and collaboration across platforms
        </p>
      </div>

      {}
          {connections.length > 0 && (
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Connected Accounts</h3>
              {connections.map((connection) => {
                const config = PLATFORM_CONFIGS[connection.platform];
                return (
                  <Card key={connection.id}>
                    <CardContent className="p-6">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-4">
                          <div className="w-10 h-10 rounded-lg flex items-center justify-center">
                            {config.icon('w-10 h-10')}
                          </div>
                          <div>
                            <div className="flex items-center space-x-2">
                              <h4 className="font-semibold">{config.name}</h4>
                              <Badge variant={connection.is_active ? "default" : "secondary"}>
                                {connection.is_active ? "Active" : "Inactive"}
                              </Badge>
                            </div>
                        <p className="text-sm text-muted-foreground">
                          {connection.username}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          Connected: {new Date(connection.connected_at).toLocaleDateString()}
                          {connection.last_sync && (
                            <span> • Last sync: {new Date(connection.last_sync).toLocaleDateString()}</span>
                          )}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => syncPlatform(connection.id, connection.platform)}
                        disabled={syncing === connection.id}
                      >
                        <RefreshCw className={`h-4 w-4 ${syncing === connection.id ? 'animate-spin' : ''}`} />
                        Sync
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => disconnectPlatform(connection.id)}
                      >
                        <Trash2 className="h-4 w-4" />
                        Disconnect
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {}
      {availablePlatforms.length > 0 && (
        <div className="space-y-4">
          <h3 className="text-lg font-semibold">Available Platforms</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {availablePlatforms.map((platform) => {
              const config = PLATFORM_CONFIGS[platform as keyof typeof PLATFORM_CONFIGS];
              return (
                <Card key={platform} className="hover:shadow-md transition-shadow">
                  <CardHeader className="pb-3">
                    <div className="flex items-center space-x-3">
                      <div className="w-8 h-8 rounded-lg flex items-center justify-center">
                        {config.icon('w-8 h-8')}
                      </div>
                      <CardTitle className="text-lg">{config.name}</CardTitle>
                    </div>
                    <CardDescription>{config.description}</CardDescription>
                  </CardHeader>
                  <CardContent className="pt-0">
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
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </div>
      )}

      {connections.length === 0 && availablePlatforms.length === 0 && (
        <Card>
          <CardContent className="p-6 text-center">
            <p className="text-muted-foreground">All available platforms are connected!</p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

export default SocialConnections;
