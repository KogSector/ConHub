'use client';

import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { GitBranch, Github, GitlabIcon as Bitbucket, AlertCircle } from 'lucide-react';

interface ConnectRepositoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function ConnectRepositoryDialog({ open, onOpenChange, onSuccess }: ConnectRepositoryDialogProps) {
  const [provider, setProvider] = useState('');
  const [name, setName] = useState('');
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  const [config, setConfig] = useState<Record<string, any>>({
    includeReadme: true,
    includeCode: true,
    defaultBranch: 'main',
    enableWebhooks: false,
    fileExtensions: ['.js', '.ts', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.md']
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnect = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await fetch('/api/data-sources/connect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          type: provider, 
          credentials, 
          config: { 
            ...config, 
            name: name || `${provider}-${Date.now()}`
          } 
        })
      });
      
      const data = await response.json();
      
      if (data.success) {
        onSuccess();
        onOpenChange(false);
        resetForm();
      } else {
        setError(data.error || 'Failed to connect repository');
      }
    } catch (error) {
      console.error('Error connecting repository:', error);
      setError('Network error occurred while connecting repository');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setProvider('');
    setName('');
    setCredentials({});
    setConfig({
      includeReadme: true,
      includeCode: true,
      defaultBranch: 'main',
      enableWebhooks: false,
      fileExtensions: ['.js', '.ts', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.md']
    });
    setError(null);
  };

  const renderCredentialFields = () => {
    switch (provider) {
      case 'github':
        return (
          <div className="space-y-4">
            <div>
              <Label htmlFor="accessToken">GitHub Access Token</Label>
              <Input
                id="accessToken"
                type="password"
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
                value={credentials.accessToken || ''}
                onChange={(e) => setCredentials({ ...credentials, accessToken: e.target.value })}
              />
              <p className="text-xs text-muted-foreground mt-1">
                Generate a token at: Settings → Developer settings → Personal access tokens
              </p>
            </div>
            <div>
              <Label htmlFor="repositories">Repositories</Label>
              <Textarea
                id="repositories"
                placeholder="owner/repo1, owner/repo2"
                value={config.repositories?.join(', ') || ''}
                onChange={(e) => setConfig({ 
                  ...config, 
                  repositories: e.target.value.split(',').map(r => r.trim()).filter(Boolean)
                })}
              />
              <p className="text-xs text-muted-foreground mt-1">
                Comma-separated list of repositories (e.g., microsoft/vscode, facebook/react)
              </p>
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
              <p className="text-xs text-muted-foreground mt-1">
                Create at: Settings → Personal Bitbucket settings → App passwords
              </p>
            </div>
            <div>
              <Label htmlFor="repositories">Repositories</Label>
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

      default:
        return null;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <GitBranch className="w-5 h-5" />
            Connect Repository
          </DialogTitle>
        </DialogHeader>
        
        <div className="space-y-6">
          {error && (
            <div className="bg-destructive/10 text-destructive text-sm p-3 rounded-md flex items-center gap-2">
              <AlertCircle className="w-4 h-4" />
              {error}
            </div>
          )}

          <div>
            <Label htmlFor="name">Connection Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My GitHub Repositories"
            />
          </div>

          <div>
            <Label htmlFor="provider">Repository Provider</Label>
            <Select value={provider} onValueChange={setProvider}>
              <SelectTrigger>
                <SelectValue placeholder="Select a repository provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="github">
                  <div className="flex items-center gap-2">
                    <Github className="w-4 h-4" />
                    GitHub
                  </div>
                </SelectItem>
                <SelectItem value="bitbucket">
                  <div className="flex items-center gap-2">
                    <Bitbucket className="w-4 h-4" />
                    BitBucket
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          {provider && renderCredentialFields()}

          {provider && (
            <div className="space-y-4 border-t pt-4">
              <h4 className="font-medium">Configuration Options</h4>
              
              <div className="grid gap-4">
                <div className="flex items-center justify-between">
                  <div>
                    <Label>Include README files</Label>
                    <p className="text-xs text-muted-foreground">Extract and index README.md files</p>
                  </div>
                  <Switch
                    checked={config.includeReadme}
                    onCheckedChange={(checked) => setConfig({ ...config, includeReadme: checked })}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label>Include source code</Label>
                    <p className="text-xs text-muted-foreground">Index source code files for search</p>
                  </div>
                  <Switch
                    checked={config.includeCode}
                    onCheckedChange={(checked) => setConfig({ ...config, includeCode: checked })}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label>Enable webhooks (future)</Label>
                    <p className="text-xs text-muted-foreground">Real-time updates when repositories change</p>
                  </div>
                  <Switch
                    checked={config.enableWebhooks}
                    onCheckedChange={(checked) => setConfig({ ...config, enableWebhooks: checked })}
                  />
                </div>

                <div>
                  <Label>Default Branch</Label>
                  <Select 
                    value={config.defaultBranch} 
                    onValueChange={(value) => setConfig({ ...config, defaultBranch: value })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="main">main</SelectItem>
                      <SelectItem value="master">master</SelectItem>
                      <SelectItem value="develop">develop</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label>File Extensions to Index</Label>
                  <div className="flex flex-wrap gap-1 mt-2">
                    {['.js', '.ts', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.md', '.txt', '.json', '.yml', '.yaml'].map((ext) => (
                      <Badge
                        key={ext}
                        variant={config.fileExtensions?.includes(ext) ? "default" : "outline"}
                        className="cursor-pointer text-xs"
                        onClick={() => {
                          const extensions = config.fileExtensions || [];
                          const newExtensions = extensions.includes(ext) 
                            ? extensions.filter(e => e !== ext)
                            : [...extensions, ext];
                          setConfig({ ...config, fileExtensions: newExtensions });
                        }}
                      >
                        {ext}
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          <div className="flex justify-end space-x-2 pt-4">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleConnect} 
              disabled={!provider || !name || loading || (!credentials.accessToken && !credentials.username)}
            >
              {loading ? 'Connecting...' : 'Connect Repository'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}