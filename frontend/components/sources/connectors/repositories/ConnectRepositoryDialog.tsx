'use client';

import { useMemo, useState, useEffect } from "react";
import type { ChangeEvent } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { dataApiClient, apiClient, ApiResponse, unwrapResponse } from '@/lib/api';
import { useAuth } from '@/hooks/use-auth';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { GitBranch, AlertCircle } from 'lucide-react';
import GitHubIcon from '@/components/icons/GitHubIcon';
import GitLabIcon from '@/components/icons/GitLabIcon';
import BitbucketIcon from '@/components/icons/BitbucketIcon';

interface ConnectRepositoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function ConnectRepositoryDialog({ open, onOpenChange, onSuccess }: ConnectRepositoryDialogProps) {
  const { token, connections } = useAuth()
  const [provider, setProvider] = useState('');
  const [name, setName] = useState('');
  const [repositoryUrl, setRepositoryUrl] = useState('');
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  interface RepositoryConfigUI {
    includeReadme: boolean;
    includeCode: boolean;
    defaultBranch: string;
    enableWebhooks: boolean;
    fileExtensions: string[];
  }
  const [config, setConfig] = useState<RepositoryConfigUI>({
    includeReadme: true,
    includeCode: true,
    defaultBranch: 'main',
    enableWebhooks: false,
    fileExtensions: ['.js', '.ts', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.md']
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [branches, setBranches] = useState<string[]>([]);
  const [isFetchingBranches, setIsFetchingBranches] = useState(false);
  const [fetchBranchesError, setFetchBranchesError] = useState<string | null>(null);
  const [showAdvancedSettings, setShowAdvancedSettings] = useState(false);
  const [isValidated, setIsValidated] = useState(false);
  const [availableFileExtensions, setAvailableFileExtensions] = useState<string[]>([]);
  const [needsSocialConnect, setNeedsSocialConnect] = useState<string | null>(null);
  const [checkingConnection, setCheckingConnection] = useState(false);
  const isProviderSelected = provider === 'github' || provider === 'gitlab' || provider === 'bitbucket';
  const hasBranches = branches.length > 0;

  // Reset form when modal closes
  useEffect(() => {
    if (!open) {
      resetForm();
    }
  }, [open]);

  const isUrlValid = useMemo(() => {
    if (!repositoryUrl) return false;
    try {
      
      new URL(repositoryUrl);
      return true;
    } catch (e) {
      
      if (repositoryUrl.startsWith('git@')) {
        return repositoryUrl.includes(':') && repositoryUrl.length > 10;
      }
      return false;
    }
  }, [repositoryUrl]);

  const ensureProviderConnected = async (): Promise<boolean> => {
    if (!isProviderSelected) return true;
    setCheckingConnection(true);
    try {
      const cached = Array.isArray(connections) ? connections : []
      const hasCached = cached.some((c) => c.platform === provider && c.is_active)
      if (hasCached) {
        setNeedsSocialConnect(null)
        return true
      }
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      const resp = await apiClient.get('/api/auth/connections', headers);
      const list = unwrapResponse<Array<{ platform: string; is_active: boolean }>>(resp) || [];
      const connected = list.some((c) => c.platform === provider && c.is_active);
      if (!connected) {
        setNeedsSocialConnect(provider);
      } else {
        setNeedsSocialConnect(null);
      }
      return connected;
    } catch {
      return false;
    } finally {
      setCheckingConnection(false);
    }
  };

  
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
    console.log('[FRONTEND] handleFetchBranches called');
    console.log('[FRONTEND] repositoryUrl:', repositoryUrl);
    console.log('[FRONTEND] provider:', provider);
    console.log('[FRONTEND] isProviderSelected:', isProviderSelected);
    
    if (!repositoryUrl || !isProviderSelected) {
      setFetchBranchesError("Please enter a repository URL and select a provider first.");
      return;
    }
    
    console.log('[FRONTEND] isUrlValid:', isUrlValid);
    const connected = await ensureProviderConnected();
    if (!connected) {
      setFetchBranchesError(`Please connect ${provider === 'github' ? 'GitHub' : provider === 'gitlab' ? 'GitLab' : 'BitBucket'} in Social Connections first.`);
      return;
    }
    if (!isUrlValid) {
      setFetchBranchesError("Please enter a valid repository URL.");
      return;
    }
    
    console.log('[FRONTEND] credentials:', credentials);
    
    setIsFetchingBranches(true);
    setFetchBranchesError(null);
    setBranches([]);
    setIsValidated(false);

    try {
      const repoCheck = await dataApiClient.post<{ provider?: string; name?: string; full_name?: string }>('/api/repositories/oauth/check', { 
        provider,
        repo_url: repositoryUrl.trim()
      }, token ? { Authorization: `Bearer ${token}` } : {})
      const repoName = repoCheck.name || repoCheck.full_name
      if (repoName) {
        setName(prev => prev || repoName)
      }

      let fetchedBranches: string[] = []
      let default_branch: string | undefined
      let file_extensions: string[] | undefined

      if (provider === 'github') {
        const slug = extractRepoName(repositoryUrl.trim())
        const gh = await dataApiClient.get<{ success?: boolean; data?: { branches?: string[] } }>(`/api/repositories/oauth/branches?provider=github&repo=${encodeURIComponent(slug)}`, token ? { Authorization: `Bearer ${token}` } : {})
        const ghBranches = gh && (gh as any).data && Array.isArray((gh as any).data.branches) ? (gh as any).data.branches as string[] : (gh as any).branches
        fetchedBranches = Array.isArray(ghBranches) ? ghBranches : []
      } else if (provider === 'bitbucket') {
        const slug = extractRepoName(repositoryUrl.trim())
        const bb = await dataApiClient.get<{ success?: boolean; data?: { branches?: string[] } }>(`/api/repositories/oauth/branches?provider=bitbucket&repo=${encodeURIComponent(slug)}`, token ? { Authorization: `Bearer ${token}` } : {})
        const bbBranches = bb && (bb as any).data && Array.isArray((bb as any).data.branches) ? (bb as any).data.branches as string[] : (bb as any).branches
        fetchedBranches = Array.isArray(bbBranches) ? bbBranches : []
      } else if (provider === 'gitlab') {
        const slug = extractRepoName(repositoryUrl.trim())
        const gl = await dataApiClient.get<{ success?: boolean; data?: { branches?: string[]; default_branch?: string } }>(`/api/repositories/oauth/branches?provider=gitlab&repo=${encodeURIComponent(slug)}`, token ? { Authorization: `Bearer ${token}` } : {})
        const glData = (gl as any).data || gl
        fetchedBranches = Array.isArray(glData?.branches) ? glData.branches : []
        default_branch = typeof glData?.default_branch === 'string' ? glData.default_branch : undefined
      }
      
      if (!fetchedBranches || fetchedBranches.length === 0) {
        setFetchBranchesError("No branches found. Please check the repository URL and permissions.");
        setBranches([]);
        setIsValidated(false);
      } else {
        setBranches(fetchedBranches);
        const inferredDefault = default_branch || (fetchedBranches.includes('main') ? 'main' : (fetchedBranches.includes('master') ? 'master' : fetchedBranches[0]))
        setConfig(prev => ({ 
          ...prev, 
          defaultBranch: inferredDefault || prev.defaultBranch,
          fileExtensions: file_extensions && file_extensions.length > 0 ? file_extensions : prev.fileExtensions
        }));
        if (file_extensions && file_extensions.length > 0) {
          setAvailableFileExtensions(file_extensions);
        }
        setIsValidated(true);
      }
    } catch (err: unknown) {
      console.error('[FRONTEND] Branch fetching error:', err);
      let errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[FRONTEND] Error message:', errorMessage);
      
      
      if (errorMessage.includes('401') || errorMessage.includes('Unauthorized')) {
        errorMessage = "Authentication failed. Please check your credentials and token permissions.";
      } else if (errorMessage.includes('404') || errorMessage.includes('Not Found')) {
        errorMessage = "Repository not found. Please check the URL and ensure you have access.";
      } else if (errorMessage.includes('403') || errorMessage.includes('Forbidden')) {
        errorMessage = "Access denied. Please check your token permissions and repository access.";
      } else if (errorMessage.includes('rate limit')) {
        errorMessage = "API rate limit exceeded. Please wait a few minutes before trying again.";
      }
      
      setFetchBranchesError(errorMessage);
      setBranches([]);
      setIsValidated(false);
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
    
    try {
      const connected = await ensureProviderConnected();
      if (!connected) {
        setError(`Please connect ${provider === 'github' ? 'GitHub' : provider === 'gitlab' ? 'GitLab' : 'BitBucket'} in Social Connections before connecting.`);
        return;
      }
      const payload = { 
        type: provider, 
        url: repositoryUrl,
        credentials, 
        config: { 
          ...config, 
          name: name || extractRepoName(repositoryUrl) || `${provider}-${Date.now()}`
        } 
      };

      const resp = await dataApiClient.post<ApiResponse>('/api/data/sources', payload);
      if (resp.success) {
        onSuccess();
        onOpenChange(false);
        resetForm();
      } else {
        const errorMessage = resp.error || 'Failed to connect repository';
        
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
    setBranches([]);
    setShowAdvancedSettings(false);
    setError(null);
  };

  const renderCredentialFields = () => {
    switch (provider) {
      case 'gitlab':
        return null;

      case 'bitbucket':
        return null;

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

          {needsSocialConnect && (
            <div className="mt-2 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
              <p className="text-sm text-yellow-800">
                {`You need to connect ${needsSocialConnect === 'github' ? 'GitHub' : needsSocialConnect === 'gitlab' ? 'GitLab' : 'BitBucket'} before proceeding.`}
              </p>
              <div className="mt-2 flex gap-2">
                <Button
                  title="Go to Social Connections"
                  aria-label="Go to Social Connections"
                  className="bg-blue-600 hover:bg-blue-700 text-white"
                  onClick={() => { window.location.href = '/dashboard/connections'; }}
                  disabled={checkingConnection}
                >
                  Go to Social Connections
                </Button>
              </div>
            </div>
          )}

          {isValidated && (
            <div className="space-y-3">
              <Label htmlFor="name" className="text-sm font-medium">Connection Name (Optional)</Label>
              <Input
                id="name"
                value={name}
                onChange={(e: ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
                placeholder={repositoryUrl ? extractRepoName(repositoryUrl) : "Auto-generated from repository URL"}
                className="mt-2"
              />
              <p className="text-xs text-muted-foreground mt-2">
                Leave empty to automatically use the repository name from the URL
              </p>
            </div>
          )}

          <div className="space-y-3">
            <Label htmlFor="provider" className="text-sm font-medium">Repository Provider</Label>
            <Select value={provider} onValueChange={setProvider}>
              <SelectTrigger className="mt-2">
                <SelectValue placeholder="Select a repository provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="github">
                  <div className="flex items-center gap-2">
                    <GitHubIcon className="w-4 h-4" />
                    GitHub
                  </div>
                </SelectItem>
                <SelectItem value="gitlab">
                  <div className="flex items-center gap-2">
                    <GitLabIcon className="w-4 h-4" />
                    GitLab
                  </div>
                </SelectItem>
                <SelectItem value="bitbucket">
                  <div className="flex items-center gap-2">
                    <BitbucketIcon className="w-4 h-4" />
                    BitBucket
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          {isProviderSelected && (
            <div className="space-y-3">
              <Label htmlFor="repositoryUrl" className="text-sm font-medium">Repository URL</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="repositoryUrl"
                  placeholder={
                    provider === 'gitlab' ? 'https://gitlab.com/user/repo.git' :
                    provider === 'bitbucket' ? 'https://bitbucket.org/user/repo.git' :
                    'https://github.com/user/repo.git'
                  }
                  value={repositoryUrl}
                  onChange={(e: ChangeEvent<HTMLInputElement>) => setRepositoryUrl(e.target.value)}
                />
                <Button
                  className="bg-blue-600 hover:bg-blue-700 text-white"
                  onClick={handleFetchBranches}
                  disabled={isFetchingBranches || !isUrlValid || !isProviderSelected}
                >
                  {isFetchingBranches ? 'Validating...' : 'Check'}
                </Button>
              </div>
              {fetchBranchesError && (
                <div className="mt-2 p-3 bg-red-50 border border-red-200 rounded-md">
                  <p className="text-sm text-red-700 flex items-center gap-2">
                    <span>{fetchBranchesError}</span>
                    {fetchBranchesError.toLowerCase().includes('please connect') && (
                      <Button
                        variant="link"
                        className="text-blue-700 px-0"
                        onClick={() => { window.location.href = '/dashboard/connections'; }}
                      >
                        Open Connections
                      </Button>
                    )}
                  </p>
                </div>
              )}
              {isValidated && branches.length > 0 && !fetchBranchesError && (
                <div className="mt-2 p-3 bg-green-50 border border-green-200 rounded-md">
                  <p className="text-sm text-green-700">✓ Successfully found {branches.length} branches</p>
                  {config.fileExtensions && config.fileExtensions.length > 0 && (
                    <p className="text-xs text-green-600 mt-1">
                      Found file types: {config.fileExtensions.slice(0, 10).join(', ')}
                      {config.fileExtensions.length > 10 && ` and ${config.fileExtensions.length - 10} more`}
                    </p>
                  )}
                </div>
              )}
            </div>
          )}

          {isProviderSelected && renderCredentialFields()}

          {isValidated && (
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
                    disabled={!hasBranches}
                  >
                    <SelectTrigger className={`mt-2 ${!hasBranches ? 'opacity-50 cursor-not-allowed' : ''}`}>
                      <SelectValue placeholder={hasBranches ? "Select branch" : "Fetch branches first"} />
                    </SelectTrigger>
                    <SelectContent>
                      {branches.map((branch) => (
                        <SelectItem key={branch} value={branch}>
                          {branch}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {!hasBranches && (
                    <p className="text-xs text-muted-foreground">
                      Click &quot;Check&quot; button above to fetch available branches
                    </p>
                  )}
                </div>

                {/* Advanced Settings */}
                <div className="space-y-3">
                  <button
                    type="button"
                    onClick={() => setShowAdvancedSettings(!showAdvancedSettings)}
                    className="flex items-center gap-2 text-sm font-medium hover:text-primary transition-colors"
                  >
                    <span className={`transform transition-transform duration-200 ${showAdvancedSettings ? 'rotate-90' : ''}`}>
                      ▶
                    </span>
                    Advanced Settings
                  </button>
                  
                  <div className={`transition-all duration-300 ease-in-out overflow-hidden ${
                    showAdvancedSettings ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'
                  }`}>
                    <div className="space-y-4 pt-2">
                      <div className="space-y-3">
                        <Label className="text-sm font-medium">File Extensions to Index</Label>
                        <div className="flex flex-wrap gap-2 mt-3">
                          {(availableFileExtensions.length > 0 ? availableFileExtensions : []).map((ext) => (
                            <Badge
                              key={ext}
                              variant={config.fileExtensions?.includes(ext) ? "default" : "outline"}
                              className="cursor-pointer text-xs"
                              onClick={() => {
                                const extensions = config.fileExtensions || [];
                                const newExtensions = extensions.includes(ext) 
                                  ? extensions.filter((x) => x !== ext)
                                  : [...extensions, ext];
                                setConfig({ ...config, fileExtensions: newExtensions });
                              }}
                            >
                              {ext}
                            </Badge>
                          ))}
                        </div>
                        <p className="text-xs text-muted-foreground">
                          Select file types to include in indexing. Click badges to toggle.
                        </p>
                      </div>
                    </div>
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
              disabled={!isValidated || !isProviderSelected || !repositoryUrl || loading}
            >
            {loading ? 'Connecting...' : 'Connect Repository'}
          </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
