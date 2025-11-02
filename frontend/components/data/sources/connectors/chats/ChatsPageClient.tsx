'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, Plus, Settings, RefreshCw, MoreHorizontal, Trash2, MessageSquare, Users, Hash } from "lucide-react";
import Link from "next/link";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/AlertDialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";
import { apiClient, ApiResponse } from '@/lib/api';

interface ChatSource {
  id: string;
  name: string;
  type: 'slack' | 'teams' | 'discord';
  workspace: string;
  channelCount: number;
  messageCount: number;
  lastSynced: string;
  status: 'active' | 'inactive' | 'syncing' | 'error';
  members: number;
}

interface DataSource {
  id: string;
  type: string;
  name: string;
  status: 'connected' | 'syncing' | 'error' | 'indexing';
  totalCount?: number;
  indexedCount?: number;
  config?: Record<string, unknown>;
}

export function ChatsPageClient() {
  const [chatSources, setChatSources] = useState<ChatSource[]>([]);
  const [loading, setLoading] = useState(true);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [sourceToDelete, setSourceToDelete] = useState<ChatSource | null>(null);
  const [showConnectDialog, setShowConnectDialog] = useState(false);

  useEffect(() => {
    fetchChatSources();
    fetchDataSources();
  }, []);

  const fetchChatSources = async () => {
    try {
      // Mock data for now - replace with actual API call
      const mockSources: ChatSource[] = [
        {
          id: '1',
          name: 'Engineering Team',
          type: 'slack',
          workspace: 'ConHub Workspace',
          channelCount: 12,
          messageCount: 2847,
          lastSynced: '2024-01-15T10:30:00Z',
          status: 'active',
          members: 25
        },
        {
          id: '2',
          name: 'Product Team',
          type: 'teams',
          workspace: 'ConHub Organization',
          channelCount: 8,
          messageCount: 1523,
          lastSynced: '2024-01-15T09:15:00Z',
          status: 'syncing',
          members: 18
        }
      ];
      setChatSources(mockSources);
    } catch (error) {
      console.error('Error fetching chat sources:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchDataSources = async () => {
    try {
      const response: ApiResponse<{ dataSources: DataSource[] }> = await apiClient.get('/api/data-sources');
      if (response.success && response.data) {
        const chatDataSources = response.data.dataSources.filter(
          ds => ds.type === 'slack' || ds.type === 'teams' || ds.type === 'discord' || ds.type === 'chats'
        );
      }
    } catch (error) {
      console.error('Error fetching data sources:', error);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
      case 'connected':
        return 'bg-green-100 text-green-800';
      case 'syncing':
      case 'indexing':
        return 'bg-blue-100 text-blue-800';
      case 'error':
        return 'bg-red-100 text-red-800';
      case 'inactive':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'slack':
        return <Hash className="h-4 w-4" />;
      case 'teams':
        return <Users className="h-4 w-4" />;
      case 'discord':
        return <MessageSquare className="h-4 w-4" />;
      default:
        return <MessageSquare className="h-4 w-4" />;
    }
  };

  const getTypeName = (type: string) => {
    switch (type) {
      case 'slack':
        return 'Slack';
      case 'teams':
        return 'Microsoft Teams';
      case 'discord':
        return 'Discord';
      default:
        return type;
    }
  };

  const handleDeleteSource = (source: ChatSource) => {
    setSourceToDelete(source);
    setDeleteDialogOpen(true);
  };

  const confirmDelete = async () => {
    if (!sourceToDelete) return;
    
    try {
      // API call to delete the source
      // await apiClient.delete(`/api/chat-sources/${sourceToDelete.id}`);
      setChatSources(prev => prev.filter(s => s.id !== sourceToDelete.id));
      setDeleteDialogOpen(false);
      setSourceToDelete(null);
    } catch (error) {
      console.error('Error deleting chat source:', error);
    }
  };

  const syncSource = async (sourceId: string) => {
    try {
      // API call to sync the source
      // await apiClient.post(`/api/chat-sources/${sourceId}/sync`);
      setChatSources(prev => 
        prev.map(s => s.id === sourceId ? { ...s, status: 'syncing' } : s)
      );
    } catch (error) {
      console.error('Error syncing chat source:', error);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-gray-900"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto px-4 py-8">
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center space-x-4">
            <Link href="/data/sources">
              <Button variant="ghost" size="sm">
                <ArrowLeft className="h-4 w-4 mr-2" />
                Back to Data Sources
              </Button>
            </Link>
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Chat Sources</h1>
              <p className="text-gray-600 mt-1">Connect and manage your chat platform integrations</p>
            </div>
          </div>
          <Button onClick={() => setShowConnectDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Connect Chat Source
          </Button>
        </div>

        <div className="grid gap-6">
          {chatSources.map((source) => (
            <Card key={source.id} className="hover:shadow-md transition-shadow">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
                <div className="flex items-center space-x-3">
                  {getTypeIcon(source.type)}
                  <div>
                    <CardTitle className="text-lg">{source.name}</CardTitle>
                    <p className="text-sm text-gray-600">{getTypeName(source.type)} • {source.workspace}</p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <Badge className={getStatusColor(source.status)}>
                    {source.status}
                  </Badge>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => syncSource(source.id)}>
                        <RefreshCw className="h-4 w-4 mr-2" />
                        Sync Now
                      </DropdownMenuItem>
                      <DropdownMenuItem>
                        <Settings className="h-4 w-4 mr-2" />
                        Settings
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem 
                        onClick={() => handleDeleteSource(source)}
                        className="text-red-600"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Remove
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-4 gap-4 text-sm">
                  <div>
                    <p className="text-gray-600">Channels</p>
                    <p className="font-medium">{source.channelCount}</p>
                  </div>
                  <div>
                    <p className="text-gray-600">Messages</p>
                    <p className="font-medium">{source.messageCount.toLocaleString()}</p>
                  </div>
                  <div>
                    <p className="text-gray-600">Members</p>
                    <p className="font-medium">{source.members}</p>
                  </div>
                  <div>
                    <p className="text-gray-600">Last Synced</p>
                    <p className="font-medium">
                      {new Date(source.lastSynced).toLocaleDateString()}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}

          {chatSources.length === 0 && (
            <Card className="text-center py-12">
              <CardContent>
                <MessageSquare className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No chat sources connected</h3>
                <p className="text-gray-600 mb-4">
                  Connect your first chat platform to start indexing conversations from Slack, Microsoft Teams, Discord, and other chat applications.
                </p>
                <Button onClick={() => setShowConnectDialog(true)}>
                  <Plus className="h-4 w-4 mr-2" />
                  Connect Chat Source
                </Button>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove Chat Source</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to remove "{sourceToDelete?.name}"? This will stop indexing messages from this chat platform and remove all associated data.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDelete} className="bg-red-600 hover:bg-red-700">
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Footer />
    </div>
  );
}