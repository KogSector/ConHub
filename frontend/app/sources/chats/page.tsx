'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, MessageSquare, Plus, RefreshCw, Trash2, Settings } from "lucide-react";
import Link from "next/link";

interface ChatSource {
  id: string;
  name: string;
  type: 'slack' | 'teams' | 'whatsapp';
  status: 'connected' | 'syncing' | 'error' | 'disconnected';
  messageCount: number;
  lastSync?: string;
  channels?: number;
}

export default function ChatsPage() {
  const [sources, setSources] = useState<ChatSource[]>([]);
  const [showConnectModal, setShowConnectModal] = useState(false);
  const [loading, setLoading] = useState(true);

  const sampleSources: ChatSource[] = [
    {
      id: "1",
      name: "Development Team",
      type: "slack",
      status: "connected",
      messageCount: 1250,
      lastSync: "5 minutes ago",
      channels: 8
    }
  ];

  useEffect(() => {
    setTimeout(() => {
      setSources(sampleSources);
      setLoading(false);
    }, 1000);
  }, []);

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'slack': return '💬';
      case 'teams': return '👥';
      case 'whatsapp': return '📱';
      default: return '💬';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected': return 'bg-green-500 shadow-lg shadow-green-500/50';
      case 'syncing': return 'bg-blue-500 shadow-lg shadow-blue-500/50';
      case 'error': return 'bg-red-500 shadow-lg shadow-red-500/50';
      default: return 'bg-gray-400';
    }
  };

  const handleConnectSlack = () => {
    // Simulate Slack OAuth
    window.open('https://slack.com/oauth/v2/authorize?client_id=your_client_id&scope=channels:read,chat:write', '_blank');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Loading chat sources...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
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
                <MessageSquare className="w-6 h-6 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">Chats</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">Total Sources</CardTitle>
              <MessageSquare className="w-4 h-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">{sources.length}</div>
              <p className="text-xs text-muted-foreground">Connected platforms</p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">Total Messages</CardTitle>
              <MessageSquare className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {sources.reduce((sum, source) => sum + source.messageCount, 0)}
              </div>
              <p className="text-xs text-muted-foreground">Indexed messages</p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">Active Sources</CardTitle>
              <MessageSquare className="w-4 h-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {sources.filter(s => s.status === 'connected').length}
              </div>
              <p className="text-xs text-muted-foreground">Currently active</p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">Total Channels</CardTitle>
              <MessageSquare className="w-4 h-4 text-purple-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {sources.reduce((sum, source) => sum + (source.channels || 0), 0)}
              </div>
              <p className="text-xs text-muted-foreground">Monitored channels</p>
            </CardContent>
          </Card>
        </div>

        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-xl font-semibold text-foreground">Chat Platforms</h2>
            <p className="text-sm text-muted-foreground">Connect and manage your chat platforms</p>
          </div>
          <Button onClick={() => setShowConnectModal(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Connect Platform
          </Button>
        </div>

        {sources.length === 0 ? (
          <Card className="bg-muted/50 border-dashed border-muted-foreground/25">
            <CardContent className="flex flex-col items-center justify-center py-12">
              <MessageSquare className="w-12 h-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium text-foreground mb-2">No chat platforms connected</h3>
              <p className="text-muted-foreground text-center mb-6 max-w-md">
                Connect your first chat platform to start indexing conversations and enable AI-powered insights.
              </p>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 w-full max-w-2xl">
                <Button variant="outline" className="h-24 flex flex-col space-y-2" onClick={handleConnectSlack}>
                  <span className="text-2xl">💬</span>
                  <span>Slack</span>
                </Button>
                <Button variant="outline" className="h-24 flex flex-col space-y-2">
                  <span className="text-2xl">👥</span>
                  <span>Microsoft Teams</span>
                </Button>
                <Button variant="outline" className="h-24 flex flex-col space-y-2">
                  <span className="text-2xl">📱</span>
                  <span>WhatsApp</span>
                </Button>
              </div>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-4">
            {sources.map((source) => (
              <Card key={source.id} className="bg-card border-border hover:bg-accent/5 transition-colors">
                <div className="flex flex-col px-6 py-4 gap-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className="text-2xl">{getTypeIcon(source.type)}</span>
                      <div>
                        <h3 className="font-semibold text-foreground">{source.name}</h3>
                        <p className="text-sm text-muted-foreground">{source.type.charAt(0).toUpperCase() + source.type.slice(1)}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${getStatusColor(source.status)}`}></div>
                      <Badge variant="outline" className="text-xs">{source.status}</Badge>
                    </div>
                  </div>

                  <div className="flex items-center gap-6 text-sm text-muted-foreground">
                    <span>{source.messageCount} messages</span>
                    {source.channels && <span>{source.channels} channels</span>}
                    {source.lastSync && <span>Last sync: {source.lastSync}</span>}
                  </div>

                  <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm">
                      <RefreshCw className={`w-4 h-4 mr-1 ${source.status === 'syncing' ? 'animate-spin' : ''}`} />
                      {source.status === 'syncing' ? 'Syncing...' : 'Sync'}
                    </Button>
                    <Button variant="outline" size="sm">
                      <Settings className="w-4 h-4 mr-1" />
                      Settings
                    </Button>
                    <Button variant="outline" size="sm" className="text-red-500 hover:text-red-700">
                      <Trash2 className="w-4 h-4 mr-1" />
                      Disconnect
                    </Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>

      <Footer />
    </div>
  );
}