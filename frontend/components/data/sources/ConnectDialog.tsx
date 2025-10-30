'use client';

import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { apiClient, ApiResponse } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';

interface ConnectDataSourceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function ConnectDataSourceDialog({ open, onOpenChange, onSuccess }: ConnectDataSourceDialogProps) {
  const [type, setType] = useState('');
  const [name, setName] = useState('');
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  const [config, setConfig] = useState<Record<string, any>>({});
  const [loading, setLoading] = useState(false);

  const handleConnect = async () => {
    setLoading(true);
    try {
      const resp = await apiClient.post<ApiResponse>('/api/data-sources/connect', { type, credentials, config: { ...config, name } });
      if (resp.success) {
        onSuccess();
        onOpenChange(false);
        resetForm();
      } else {
        console.error('Error connecting data source:', resp.error || resp.message);
      }
    } catch (error) {
      console.error('Error connecting data source:', error);
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setType('');
    setName('');
    setCredentials({});
    setConfig({});
  };

  const renderCredentialFields = () => {
    switch (type) {
      case 'github':
        return (
          <div className="space-y-4">
            <div>
              <Label htmlFor="accessToken">Access Token</Label>
              <Input
                id="accessToken"
                type="password"
                value={credentials.accessToken || ''}
                onChange={(e) => setCredentials({ ...credentials, accessToken: e.target.value })}
              />
            </div>
            <div>
              <Label htmlFor="repositories">Repositories (comma-separated)</Label>
              <Textarea
                id="repositories"
                placeholder="owner/repo1, owner/repo2"
                value={config.repositories?.join(', ') || ''}
                onChange={(e) => setConfig({ 
                  ...config, 
                  repositories: e.target.value.split(',').map(r => r.trim()).filter(Boolean)
                })}
              />
            </div>
          </div>
        );
      
      case 'bitbucket':
        return (
          <div className="space-y-4">
            <div>
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                value={credentials.username || ''}
                onChange={(e) => setCredentials({ ...credentials, username: e.target.value })}
              />
            </div>
            <div>
              <Label htmlFor="appPassword">App Password</Label>
              <Input
                id="appPassword"
                type="password"
                value={credentials.appPassword || ''}
                onChange={(e) => setCredentials({ ...credentials, appPassword: e.target.value })}
              />
            </div>
            <div>
              <Label htmlFor="repositories">Repositories (comma-separated)</Label>
              <Textarea
                id="repositories"
                placeholder="workspace/repo1, workspace/repo2"
                value={config.repositories?.join(', ') || ''}
                onChange={(e) => setConfig({ 
                  ...config, 
                  repositories: e.target.value.split(',').map(r => r.trim()).filter(Boolean)
                })}
              />
            </div>
          </div>
        );

      case 'url':
        return (
          <div className="space-y-4">
            <div>
              <Label htmlFor="urls">URLs (comma-separated)</Label>
              <Textarea
                id="urls"
                placeholder="https://example.com, https://docs.example.com"
                value={config.urls?.join(', ') || ''}
                onChange={(e) => setConfig({ 
                  ...config, 
                  urls: e.target.value.split(',').map(u => u.trim()).filter(Boolean)
                })}
              />
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Connect Data Source</DialogTitle>
        </DialogHeader>
        
        <div className="space-y-4">
          <div>
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Data Source"
            />
          </div>

          <div>
            <Label htmlFor="type">Type</Label>
            <Select value={type} onValueChange={setType}>
              <SelectTrigger>
                <SelectValue placeholder="Select data source type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="github">GitHub</SelectItem>
                <SelectItem value="bitbucket">BitBucket</SelectItem>
                <SelectItem value="google-drive">Google Drive</SelectItem>
                <SelectItem value="notion">Notion</SelectItem>
                <SelectItem value="url">URLs</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {renderCredentialFields()}

          <div className="flex justify-end space-x-2">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleConnect} 
              disabled={!type || !name || loading}
            >
              {loading ? 'Connecting...' : 'Connect'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
