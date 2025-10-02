'use client';

import { useToast } from "@/hooks/use-toast";
import { useMemo, useState } from "react";
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
  const [repositoryUrl, setRepositoryUrl] = useState('');
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
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [branches, setBranches] = useState<string[]>([]);
  const [selectedBranch, setSelectedBranch] = useState('');
  const [isFetchingBranches, setIsFetchingBranches] = useState(false);
  const [fetchBranchesError, setFetchBranchesError] = useState<string | null>(null);

  const isUrlValid = useMemo(() => {
    if (!repositoryUrl) return false;
    try {
      // Basic check for URL structure. More complex git URLs are also handled.
      new URL(repositoryUrl);
      return true;
    } catch (e) {
      // Handle git@... URLs which are not standard URLs
      if (repositoryUrl.startsWith('git@')) {
        return repositoryUrl.includes(':') && repositoryUrl.length > 10;
      }
      return false;
    }
  }, [repositoryUrl]);

  // Extract repository name from URL
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

  const handleFetchBranches = async () => {
    if (!repositoryUrl) {
      setFetchBranchesError("Please enter a repository URL first.");
      return;
    }
    setIsFetchingBranches(true);
    setFetchBranchesError(null);
    setBranches([]);

    try {
      const response = await fetch('/api/repositories/fetch-branches', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          repoUrl: repositoryUrl,
          credentials: credentials 
        }),
      });

      if (!response.ok) {
        const { error } = await response.json();
        throw new Error(error || "Failed to fetch branches.");
      }

      const { branches: fetchedBranches, defaultBranch } = await response.json();
      
      if (fetchedBranches.length === 0) {
        setFetchBranchesError("No branches found. Please check the repository URL and permissions.");
        setBranches(['main']); // fallback
        setConfig(prev => ({ ...prev, defaultBranch: 'main' }));
      } else {
        setBranches(fetchedBranches);
        setConfig(prev => ({ ...prev, defaultBranch: defaultBranch || fetchedBranches[0] }));
      }
    } catch (err: any) {
      setFetchBranchesError(err.message);
      setBranches(['main']); // fallback
      setConfig(prev => ({ ...prev, defaultBranch: 'main' }));
    } finally {
      setIsFetchingBranches(false);
    }
  };

  const handleCancel = () => {
    resetForm();
    onOpenChange(false);
  };

  const handleConnect = async () => {
    setLoading(true);
    setError(null);
    
    console.log('Connecting repository...', { provider, repositoryUrl, credentials: credentials ? 'present' : 'missing' });
    
    try {
      const payload = { 
        type: provider, 
        url: repositoryUrl,
        credentials, 
        config: { 
          ...config, 
          name: name || extractRepoName(repositoryUrl) || `${provider}-${Date.now()}`
        } 
      };
      
      console.log('Sending payload:', { ...payload, credentials: credentials ? 'present' : 'missing' });
      
      const response = await fetch('/api/data-sources/connect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
      
      console.log('Response status:', response.status);
      const data = await response.json();
      console.log('Response data:', data);
      
      if (data.success) {
        onSuccess();
        onOpenChange(false);
        resetForm();
      } else {
        // Provide more specific error handling
        const errorMessage = data.error || 'Failed to connect repository';
        
        if (errorMessage.includes('token')) {
          setError(`${errorMessage}\n\nPlease check:\n• Token is not expired\n• Token has correct permissions\n• For public repos: use 'public_repo' scope\n• For private repos: use 'repo' scope`);
        } else if (errorMessage.includes('not found') || errorMessage.includes('Access denied')) {
          setError(`${errorMessage}\n\nPlease verify:\n• Repository URL is correct\n• Repository exists and is accessible\n• Token has access to the repository\n• Repository is not private (if using public_repo scope)`);
        } else if (errorMessage.includes('rate limit')) {
          setError(`${errorMessage}\n\nGitHub API rate limit exceeded. Please wait a few minutes before trying again.`);
        } else {
          setError(errorMessage);
        }
      }
    } catch (error) {
      console.error('Error connecting repository:', error);
      setError('Network error occurred while connecting repository. Please check if all services are running and try again.');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setProvider('');
    setName('');
    setRepositoryUrl('');
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
          <div className="space-y-6">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <Label htmlFor="accessToken" className="text-sm font-medium">GitHub Access Token</Label>
                <a 
                  href="https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="text-xs text-blue-600 hover:text-blue-800 underline"
                >
                  How to create a token?
                </a>
              </div>
              <Input
                id="accessToken"
                type="password"
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx or github_pat_xxxxxxxxxx"
                value={credentials.accessToken || ''}
                onChange={(e) => setCredentials({ ...credentials, accessToken: e.target.value })}
                className="mt-2"
              />
              <div className="space-y-2 text-xs text-muted-foreground">
                <p className="font-medium">Token Requirements:</p>
                <ul className="space-y-1 ml-4">
                  <li>• <strong>Classic tokens (ghp_*):</strong> Need <code>repo</code> scope for private repos, <code>public_repo</code> for public repos</li>
                  <li>• <strong>Fine-grained tokens (github_pat_*):</strong> Need repository access and Contents/Metadata permissions</li>
                  <li>• <strong>Token must not be expired</strong></li>
                  <li>• <strong>Account must have access to the repository</strong></li>
                </ul>
                <div className="flex gap-4 pt-2">
                  <a 
                    href="https://github.com/settings/tokens" 
                    target="_blank" 
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:text-blue-800 underline"
                  >
                    Manage Classic Tokens
                  </a>
                  <a 
                    href="https://github.com/settings/personal-access-tokens/new" 
                    target="_blank" 
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:text-blue-800 underline"
                  >
                    Create Fine-grained Token
                  </a>
                </div>
                <div className="mt-3 p-3 bg-muted/50 rounded-md">
                  <p className="font-medium text-foreground">Quick Setup:</p>
                  <ol className="mt-1 space-y-1 ml-4">
                    <li>1. Click "Manage Classic Tokens" above</li>
                    <li>2. Generate new token (classic)</li>
                    <li>3. Select <code>public_repo</code> or <code>repo</code> scope</li>
                    <li>4. Copy the token and paste it above</li>
                  </ol>
                </div>
                <div className="mt-3 p-3 bg-blue-50 border border-blue-200 rounded-md">
                  <div className="flex items-start gap-2">
                    <div className="text-blue-600 mt-0.5">ℹ️</div>
                    <div>
                      <p className="font-medium text-blue-800">Organization Repositories</p>
                      <p className="text-xs text-blue-700 mt-1">
                        For enhanced organization access, consider using GitHub Apps instead of Personal Access Tokens. 
                        GitHub Apps provide better security, audit trails, and fine-grained permissions for organization repositories.
                      </p>
                      <p className="text-xs text-blue-600 mt-2">
                        <strong>Personal tokens work for:</strong> Personal repos, org repos with user access<br/>
                        <strong>GitHub Apps work best for:</strong> Organization-wide access, team repositories, enterprise use
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        );

      case 'bitbucket':
        return (
          <div className="space-y-6">
            <div className="space-y-3">
              <Label htmlFor="username" className="text-sm font-medium">Username</Label>
              <Input
                id="username"
                value={credentials.username || ''}
                onChange={(e) => setCredentials({ ...credentials, username: e.target.value })}
                className="mt-2"
              />
            </div>
            <div className="space-y-3">
              <Label htmlFor="appPassword" className="text-sm font-medium">App Password</Label>
              <Input
                id="appPassword"
                type="password"
                value={credentials.appPassword || ''}
                onChange={(e) => setCredentials({ ...credentials, appPassword: e.target.value })}
                className="mt-2"
              />
              <p className="text-xs text-muted-foreground mt-2">
                Create at: Settings → Personal Bitbucket settings → App passwords
              </p>
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
        
        <div className="space-y-8">
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

          <div className="space-y-3">
            <Label htmlFor="name" className="text-sm font-medium">Connection Name (Optional)</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={repositoryUrl ? extractRepoName(repositoryUrl) : "Auto-generated from repository URL"}
              className="mt-2"
            />
            <p className="text-xs text-muted-foreground mt-2">
              Leave empty to automatically use the repository name from the URL
            </p>
          </div>

          <div className="space-y-3">
            <Label htmlFor="provider" className="text-sm font-medium">Repository Provider</Label>
            <Select value={provider} onValueChange={setProvider}>
              <SelectTrigger className="mt-2">
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

          {provider && (
            <div className="space-y-3">
              <Label htmlFor="repositoryUrl" className="text-sm font-medium">Repository URL</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="repositoryUrl"
                  placeholder="https://github.com/user/repo.git"
                  value={repositoryUrl}
                  onChange={(e) => setRepositoryUrl(e.target.value)}
                />
                <Button
                  variant="outline"
                  onClick={handleFetchBranches}
                  disabled={isFetchingBranches || !isUrlValid}
                >
                  {isFetchingBranches ? 'Checking...' : 'Check'}
                </Button>
              </div>
              {fetchBranchesError && <p className="text-sm text-red-500 mt-2">{fetchBranchesError}</p>}
            </div>
          )}

          {provider && renderCredentialFields()}

          {provider && (
            <div className="space-y-6 border-t pt-6">
              <h4 className="font-medium text-base">Configuration Options</h4>
              
              <div className="grid gap-6">
                <div className="flex items-center justify-between">
                  <div className="space-y-1">
                    <Label className="text-sm font-medium">Include README files</Label>
                    <p className="text-xs text-muted-foreground">Extract and index README.md files</p>
                  </div>
                  <Switch
                    checked={config.includeReadme}
                    onCheckedChange={(checked) => setConfig({ ...config, includeReadme: checked })}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div className="space-y-1">
                    <Label className="text-sm font-medium">Include source code</Label>
                    <p className="text-xs text-muted-foreground">Index source code files for search</p>
                  </div>
                  <Switch
                    checked={config.includeCode}
                    onCheckedChange={(checked) => setConfig({ ...config, includeCode: checked })}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div className="space-y-1">
                    <Label className="text-sm font-medium">Enable webhooks (future)</Label>
                    <p className="text-xs text-muted-foreground">Real-time updates when repositories change</p>
                  </div>
                  <Switch
                    checked={config.enableWebhooks}
                    onCheckedChange={(checked) => setConfig({ ...config, enableWebhooks: checked })}
                  />
                </div>

                <div className="space-y-3">
                  <Label className="text-sm font-medium">Default Branch</Label>
                  <Select 
                    value={config.defaultBranch} 
                    onValueChange={(value) => setConfig({ ...config, defaultBranch: value })}
                  >
                    <SelectTrigger className="mt-2">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="main">main</SelectItem>
                      <SelectItem value="master">master</SelectItem>
                      <SelectItem value="develop">develop</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-3">
                  <Label className="text-sm font-medium">File Extensions to Index</Label>
                  <div className="flex flex-wrap gap-2 mt-3">
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

          <div className="flex justify-end space-x-3 pt-6">
            <Button variant="outline" onClick={handleCancel}>
              Cancel
            </Button>
            <Button 
              onClick={handleConnect} 
              disabled={!provider || !repositoryUrl || loading || (!credentials.accessToken && !credentials.username)}
            >
              {loading ? 'Connecting...' : 'Connect Repository'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}