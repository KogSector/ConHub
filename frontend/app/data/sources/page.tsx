'use client';

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Plus, RefreshCw, Trash2 } from 'lucide-react';
import { ConnectDataSourceDialog } from '@/components/data/sources/ConnectDialog';

interface DataSource {
  id: string;
  type: string;
  name: string;
  status: 'connected' | 'disconnected' | 'syncing' | 'error' | 'indexing';
  createdAt: string;
  lastSyncAt?: string;
  indexedCount?: number;
  totalCount?: number;
  error?: string;
}

export default function DataSourcesPage() {
  const [dataSources, setDataSources] = useState<DataSource[]>([]);
  const [loading, setLoading] = useState(true);
  const [showConnectDialog, setShowConnectDialog] = useState(false);

  useEffect(() => {
    fetchDataSources();
  }, []);

  const fetchDataSources = async () => {
    try {
      const response = await fetch('/api/data-sources');
      const data = await response.json();
      if (data.success) {
        setDataSources(data.dataSources);
      }
    } catch (error) {
      console.error('Error fetching data sources:', error);
    } finally {
      setLoading(false);
    }
  };

  const syncDataSource = async (sourceId: string) => {
    try {
      const response = await fetch(`/api/data-sources/${sourceId}/sync`, {
        method: 'POST'
      });
      const data = await response.json();
      if (data.success) {
        fetchDataSources();
      }
    } catch (error) {
      console.error('Error syncing data source:', error);
    }
  };

  const deleteDataSource = async (sourceId: string) => {
    if (!confirm('Are you sure you want to delete this data source?')) return;
    
    try {
      const response = await fetch(`/api/data-sources/${sourceId}`, {
        method: 'DELETE'
      });
      const data = await response.json();
      if (data.success) {
        fetchDataSources();
      }
    } catch (error) {
      console.error('Error deleting data source:', error);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected': return 'bg-green-500';
      case 'syncing': case 'indexing': return 'bg-blue-500';
      case 'error': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'github': return '🐙';
      case 'bitbucket': return '🪣';
      case 'google-drive': return '📁';
      case 'notion': return '📝';
      case 'url': return '🌐';
      default: return '📄';
    }
  };

  if (loading) {
    return <div className="flex justify-center p-8">Loading...</div>;
  }

  return (
    <div className="container mx-auto p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">Data Sources</h1>
        <Button onClick={() => setShowConnectDialog(true)}>
          <Plus className="w-4 h-4 mr-2" />
          Connect Data Source
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {dataSources.map((source) => (
          <Card key={source.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium flex items-center">
                <span className="mr-2">{getTypeIcon(source.type)}</span>
                {source.name}
              </CardTitle>
              <Badge className={getStatusColor(source.status)}>
                {source.status}
              </Badge>
            </CardHeader>
            <CardContent>
              <div className="text-xs text-muted-foreground mb-4">
                <p>Type: {source.type}</p>
                <p>Created: {new Date(source.createdAt).toLocaleDateString()}</p>
                {source.lastSyncAt && (
                  <p>Last sync: {new Date(source.lastSyncAt).toLocaleDateString()}</p>
                )}
                {source.indexedCount !== undefined && source.totalCount !== undefined && (
                  <p>Indexed: {source.indexedCount}/{source.totalCount}</p>
                )}
                {source.error && (
                  <p className="text-red-500">Error: {source.error}</p>
                )}
              </div>
              
              <div className="flex space-x-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => syncDataSource(source.id)}
                  disabled={source.status === 'syncing' || source.status === 'indexing'}
                >
                  <RefreshCw className="w-3 h-3 mr-1" />
                  Sync
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => deleteDataSource(source.id)}
                >
                  <Trash2 className="w-3 h-3 mr-1" />
                  Delete
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {dataSources.length === 0 && (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">No data sources connected yet.</p>
          <Button onClick={() => setShowConnectDialog(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Connect Your First Data Source
          </Button>
        </div>
      )}

      <ConnectDataSourceDialog
        open={showConnectDialog}
        onOpenChange={setShowConnectDialog}
        onSuccess={fetchDataSources}
      />
    </div>
  );
}