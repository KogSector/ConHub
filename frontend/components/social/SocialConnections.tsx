"use client"

import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Trash2, RefreshCw, Plus } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { securityApiClient, apiClient, unwrapResponse } from '@/lib/api';

interface SocialConnection {
  id: string;
  platform: 'slack' | 'notion' | 'google_drive' | 'gmail' | 'dropbox' | 'linkedin' | 'github' | 'bitbucket' | 'gitlab';
  username: string;
  is_active: boolean;
  connected_at: string;
  last_sync: string | null;
}

const PLATFORM_CONFIGS = {
  slack: {
    name: 'Slack',
    description: 'Connect to sync messages and channels',
    color: 'bg-purple-500',
    icon: '💬'
  },
  notion: {
    name: 'Notion',
    description: 'Sync pages and databases',
    color: 'bg-gray-800',
    icon: '📝'
  },
  google_drive: {
    name: 'Google Drive',
    description: 'Access files and documents',
    color: 'bg-blue-500',
    icon: '💾'
  },
  gmail: {
    name: 'Gmail',
    description: 'Sync email conversations',
    color: 'bg-red-500',
    icon: '📧'
  },
  dropbox: {
    name: 'Dropbox',
    description: 'Sync files and folders',
    color: 'bg-blue-600',
    icon: '📁'
  },
  linkedin: {
    name: 'LinkedIn',
    description: 'Connect professional network',
    color: 'bg-blue-700',
    icon: '👔'
  }
  ,
  github: {
    name: 'GitHub',
    description: 'Connect your GitHub account',
    color: 'bg-gray-900',
    icon: '🐙'
  },
  bitbucket: {
    name: 'Bitbucket',
    description: 'Connect your Bitbucket account',
    color: 'bg-blue-800',
    icon: '🧩'
  }
  ,
  gitlab: {
    name: 'GitLab',
    description: 'Connect your GitLab account',
    color: 'bg-orange-600',
    icon: '🦊'
  }
};

export function SocialConnections() {
  const [connections, setConnections] = useState<SocialConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState<string | null>(null);
  const { toast } = useToast();

  const fetchConnections = useCallback(async () => {
    try {
      const resp = await securityApiClient.get('/api/security/connections');

      const data = unwrapResponse<SocialConnection[]>(resp) ?? []
      setConnections(data)
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
  }, [toast]);

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
      if (platform === 'github' || platform === 'bitbucket' || platform === 'gitlab') {
        const resp = await apiClient.get<{ url: string; state: string }>(`/api/auth/oauth/url?provider=${platform}`)
        const { url: authUrl } = resp
        if (authUrl) {
          window.open(authUrl, '_blank', 'width=500,height=700')
          setTimeout(() => { fetchConnections(); }, 5000)
        } else {
          toast({ title: 'Error', description: `Failed to get ${platform} auth URL`, variant: 'destructive' })
        }
        return
      }
      const resp = await securityApiClient.post('/api/security/connections/connect', { platform });
      const payload = unwrapResponse<{ account?: { credentials?: { auth_url?: string } } }>(resp) ?? {}
      const authUrl = payload?.account?.credentials?.auth_url
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
      toast({ title: 'Error', description: `Failed to connect to ${platform}`, variant: 'destructive' });
    }
  };

  const disconnectPlatform = async (connectionId: string) => {
    try {
      await securityApiClient.delete(`/api/security/connections/${connectionId}`);
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
      <div>
        <h2 className="text-2xl font-bold mb-2">Connections</h2>
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
                      <div className={`w-10 h-10 ${config.color} rounded-lg flex items-center justify-center text-white text-lg`}>
                        {config.icon}
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
                      <div className={`w-8 h-8 ${config.color} rounded-lg flex items-center justify-center text-white`}>
                        {config.icon}
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
