'use client';

import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import { AlertCircle, Github, Key, Zap, Shield, Users, Check } from 'lucide-react';

interface AdvancedRepositoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function AdvancedRepositoryDialog({ open, onOpenChange, onSuccess }: AdvancedRepositoryDialogProps) {
  const [authMethod, setAuthMethod] = useState<'token' | 'oauth' | 'app'>('token');
  const [provider, setProvider] = useState('github');
  const [repositoryUrl, setRepositoryUrl] = useState('');
  const [accessToken, setAccessToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [installations, setInstallations] = useState<any[]>([]);
  const [selectedInstallation, setSelectedInstallation] = useState('');

  const extractRepoName = (url: string): string => {
    try {
      const urlObj = new URL(url);
      const pathParts = urlObj.pathname.split('/').filter(Boolean);
      if (pathParts.length >= 2) {
        const owner = pathParts[pathParts.length - 2];
        const repo = pathParts[pathParts.length - 1].replace('.git', '');
        return `${owner}/${repo}`;
      }
      return url;
    } catch {
      return url;
    }
  };

  const handleOAuthLogin = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const state = Math.random().toString(36).substring(7);
      const response = await fetch('/api/auth/github/url?state=' + state);
      const data = await response.json();
      
      if (data.success) {
        
        const popup = window.open(
          data.oauthUrl,
          'github-oauth',
          'width=600,height=700,scrollbars=yes,resizable=yes'
        );
        
        
        const checkClosed = setInterval(() => {
          if (popup?.closed) {
            clearInterval(checkClosed);
            setLoading(false);
            
          }
        }, 1000);
      } else {
        setError(data.error || 'Failed to initiate OAuth');
      }
    } catch (error) {
      console.error('OAuth initialization failed:', error);
      setError('Failed to start OAuth process');
    } finally {
      setLoading(false);
    }
  };

  const loadInstallations = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/auth/github/installations');
      const data = await response.json();
      
      if (data.success) {
        setInstallations(data.installations);
      } else {
        setError(data.error || 'Failed to load installations');
      }
    } catch (error) {
      console.error('Failed to load installations:', error);
      setError('Failed to load GitHub App installations');
    } finally {
      setLoading(false);
    }
  };

  const handleConnect = async () => {
    setLoading(true);
    setError(null);
    
    try {
      let credentials: any = {};
      
      switch (authMethod) {
        case 'token':
          credentials = {
            accessToken,
            authMethod: { type: 'token' }
          };
          break;
        case 'oauth':
          credentials = {
            accessToken,
            authMethod: { type: 'oauth' }
          };
          break;
        case 'app':
          credentials = {
            authMethod: { 
              type: 'app', 
              installationId: selectedInstallation 
            }
          };
          break;
      }
      
      const payload = {
        type: provider,
        url: repositoryUrl,
        credentials,
        config: {
          name: extractRepoName(repositoryUrl),
          includeReadme: true,
          includeCode: true,
          defaultBranch: 'main'
        }
      };
      
      const response = await fetch('/api/data-sources/connect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
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
      console.error('Connection failed:', error);
      setError('Network error occurred while connecting repository');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setAuthMethod('token');
    setProvider('github');
    setRepositoryUrl('');
    setAccessToken('');
    setSelectedInstallation('');
    setError(null);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Github className="w-5 h-5" />
            Advanced Repository Connection
          </DialogTitle>
        </DialogHeader>
        
        <div className="space-y-6">
          {error && (
            <div className="bg-destructive/10 text-destructive text-sm p-4 rounded-md border border-destructive/20">
              <div className="flex items-start gap-2">
                <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
                <div className="space-y-2">
                  <div className="font-medium">Connection Failed</div>
                  <div className="whitespace-pre-line">{error}</div>
                </div>
              </div>
            </div>
          )}

          <Tabs value={authMethod} onValueChange={(value: any) => setAuthMethod(value)}>
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="token" className="flex items-center gap-2">
                <Key className="w-4 h-4" />
                Personal Token
              </TabsTrigger>
              <TabsTrigger value="oauth" className="flex items-center gap-2">
                <Shield className="w-4 h-4" />
                OAuth Login
              </TabsTrigger>
              <TabsTrigger value="app" className="flex items-center gap-2">
                <Zap className="w-4 h-4" />
                GitHub App
              </TabsTrigger>
            </TabsList>

            <TabsContent value="token" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Key className="w-4 h-4" />
                    Personal Access Token
                  </CardTitle>
                  <CardDescription>
                    Use your personal GitHub token for repository access
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label htmlFor="repository-url">Repository URL</Label>
                    <Input
                      id="repository-url"
                      value={repositoryUrl}
                      onChange={(e) => setRepositoryUrl(e.target.value)}
                      placeholder="https://github.com/owner/repo"
                    />
                  </div>
                  <div>
                    <Label htmlFor="access-token">GitHub Access Token</Label>
                    <Input
                      id="access-token"
                      type="password"
                      value={accessToken}
                      onChange={(e) => setAccessToken(e.target.value)}
                      placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
                    />
                    <div className="mt-2 text-xs text-muted-foreground">
                      <p><strong>Required Scopes:</strong> For private repos use <code>repo</code>, for public repos use <code>public_repo</code></p>
                      <div className="flex gap-2 mt-1">
                        <a href="https://github.com/settings/tokens" target="_blank" className="text-blue-600 hover:underline">
                          Manage Tokens
                        </a>
                        <span>•</span>
                        <a href="https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token" target="_blank" className="text-blue-600 hover:underline">
                          How to Create
                        </a>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="oauth" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Shield className="w-4 h-4" />
                    OAuth Authentication
                  </CardTitle>
                  <CardDescription>
                    Securely connect using GitHub OAuth (recommended for private repos)
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center gap-4 p-4 bg-muted/50 rounded-lg">
                    <div className="flex-1">
                      <h4 className="font-medium">Benefits of OAuth:</h4>
                      <ul className="text-sm text-muted-foreground mt-1 space-y-1">
                        <li>• Automatic access to all your repositories</li>
                        <li>• No need to manage tokens manually</li>
                        <li>• Secure, revokable access</li>
                        <li>• Works with private and organization repos</li>
                      </ul>
                    </div>
                    <Button onClick={handleOAuthLogin} disabled={loading}>
                      <Github className="w-4 h-4 mr-2" />
                      {loading ? 'Connecting...' : 'Login with GitHub'}
                    </Button>
                  </div>
                  <div>
                    <Label htmlFor="oauth-repository-url">Repository URL</Label>
                    <Input
                      id="oauth-repository-url"
                      value={repositoryUrl}
                      onChange={(e) => setRepositoryUrl(e.target.value)}
                      placeholder="https://github.com/owner/repo"
                    />
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="app" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Zap className="w-4 h-4" />
                    GitHub App Integration
                  </CardTitle>
                  <CardDescription>
                    Enterprise-grade integration for organizations
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center gap-4 p-4 bg-muted/50 rounded-lg">
                    <div className="flex-1">
                      <h4 className="font-medium">GitHub App Features:</h4>
                      <ul className="text-sm text-muted-foreground mt-1 space-y-1">
                        <li>• Organization-wide repository access</li>
                        <li>• Fine-grained permissions</li>
                        <li>• No personal token required</li>
                        <li>• Audit logs and compliance</li>
                      </ul>
                    </div>
                    <Button onClick={loadInstallations} disabled={loading}>
                      <Users className="w-4 h-4 mr-2" />
                      {loading ? 'Loading...' : 'Load Installations'}
                    </Button>
                  </div>
                  
                  {installations.length > 0 && (
                    <div>
                      <Label>Select Installation</Label>
                      <Select value={selectedInstallation} onValueChange={setSelectedInstallation}>
                        <SelectTrigger>
                          <SelectValue placeholder="Choose an installation" />
                        </SelectTrigger>
                        <SelectContent>
                          {installations.map((installation) => (
                            <SelectItem key={installation.id} value={installation.id.toString()}>
                              <div className="flex items-center gap-2">
                                <Badge variant={installation.account.type === 'Organization' ? 'default' : 'secondary'}>
                                  {installation.account.type}
                                </Badge>
                                {installation.account.login}
                              </div>
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  )}
                  
                  <div>
                    <Label htmlFor="app-repository-url">Repository URL</Label>
                    <Input
                      id="app-repository-url"
                      value={repositoryUrl}
                      onChange={(e) => setRepositoryUrl(e.target.value)}
                      placeholder="https://github.com/owner/repo"
                    />
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

          <div className="flex justify-end space-x-3 pt-4 border-t">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleConnect} 
              disabled={loading || !repositoryUrl || (authMethod === 'token' && !accessToken) || (authMethod === 'app' && !selectedInstallation)}
            >
              {loading ? 'Connecting...' : 'Connect Repository'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}