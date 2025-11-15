'use client';

import { useMemo, useState, useEffect } from "react";
import type { ChangeEvent } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { dataApiClient, apiClient, ApiResponse } from '@/lib/api';
import { useAuth } from '@/contexts/auth-context';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { GitBranch, Github, GitlabIcon, AlertCircle } from 'lucide-react';
import { BitbucketIcon } from '@/components/icons/BitbucketIcon';

interface ConnectRepositoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function ConnectRepositoryDialog({ open, onOpenChange, onSuccess }: ConnectRepositoryDialogProps) {
  const { token } = useAuth()
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
    if (!isUrlValid) {
      setFetchBranchesError("Please enter a valid repository URL.");
      return;
    }
    
    console.log('[FRONTEND] credentials:', credentials);
    if ((provider === 'github' || provider === 'gitlab') && !credentials.accessToken) {
      setFetchBranchesError(`Please enter your ${provider === 'github' ? 'GitHub' : 'GitLab'} access token first.`);
      return;
    }
    if (provider === 'bitbucket' && (!credentials.username || !credentials.appPassword)) {
      setFetchBranchesError("Please enter your Bitbucket username and app password first.");
      return;
    }
    
    
    if (provider === 'github' && credentials.accessToken) {
      const token = credentials.accessToken.trim();
      if (!token.startsWith('ghp_') && !token.startsWith('github_pat_')) {
        setFetchBranchesError("GitHub token should start with 'ghp_' (classic) or 'github_pat_' (fine-grained).");
        return;
      }
    }
    if (provider === 'gitlab' && credentials.accessToken) {
      const token = credentials.accessToken.trim();
      if (!token.startsWith('glpat-')) {
        setFetchBranchesError("GitLab token should start with 'glpat-'.");
        return;
      }
    }
    
    setIsFetchingBranches(true);
    setFetchBranchesError(null);
    setBranches([]);

    try {
      console.log('[FRONTEND] Building credentials payload');
      let credentialsPayload: Record<string, string> | undefined;
      if (provider === 'github' || provider === 'gitlab') {
        credentialsPayload = {
          accessToken: credentials.accessToken.trim()
        };
      } else if (provider === 'bitbucket') {
        credentialsPayload = {
          username: credentials.username.trim(),
          appPassword: credentials.appPassword.trim()
        };
      }
      
      const repoCheck = await apiClient.post<{ provider?: string; name?: string; full_name?: string }>('/api/auth/repos/check', { 
        provider,
        repo_url: repositoryUrl.trim(),
        access_token: credentialsPayload?.accessToken
      }, token ? { Authorization: `Bearer ${token}` } : {})
      const repoName = repoCheck.name || repoCheck.full_name
      if (repoName) {
        setName(prev => prev || repoName)
      }

      let fetchedBranches: string[] = []
      let default_branch: string | undefined
      let file_extensions: string[] | undefined
      if (provider === 'github') {
        const urlObj = new URL(repositoryUrl.trim())
        const parts = urlObj.pathname.split('/').filter(Boolean)
        const owner = parts[0]
        const repo = parts[1]?.replace('.git','')
        const res = await apiClient.get<{ branches: string[] }>(`/api/auth/repos/github/branches?repo=${owner}/${repo}`, token ? { Authorization: `Bearer ${token}` } : {})
        fetchedBranches = res.branches || []
      } else if (provider === 'bitbucket') {
        const urlObj = new URL(repositoryUrl.trim())
        const parts = urlObj.pathname.split('/').filter(Boolean)
        const workspace = parts[0]
        const repo = parts[1]?.replace('.git','')
        const res = await apiClient.get<{ branches: string[] }>(`/api/auth/repos/bitbucket/branches?repo=${workspace}/${repo}`, token ? { Authorization: `Bearer ${token}` } : {})
        fetchedBranches = res.branches || []
      } else {
        const requestPayload = { repoUrl: repositoryUrl.trim(), credentials: credentialsPayload };
        const resp = await dataApiClient.post<ApiResponse<{ branches: string[]; default_branch?: string; file_extensions?: string[] }>>('/api/data/sources/branches', requestPayload);
        if (!resp.success) throw new Error(resp.error || 'Failed to fetch branches.')
        const payload: { branches: string[]; default_branch?: string; file_extensions?: string[] } = resp.data ?? { branches: [], default_branch: undefined, file_extensions: undefined }
        fetchedBranches = payload.branches
        default_branch = payload.default_branch
        file_extensions = payload.file_extensions
      }
      console.log('[FRONTEND] Parsed response data:', { fetchedBranches, default_branch, file_extensions });
      
      if (!fetchedBranches || fetchedBranches.length === 0) {
        setFetchBranchesError("No branches found. Please check the repository URL and permissions.");
        setBranches(['main', 'master']); 
        setConfig(prev => ({ ...prev, defaultBranch: 'main' }));
      } else {
        setBranches(fetchedBranches);
        setConfig(prev => ({ 
          ...prev, 
          defaultBranch: default_branch || fetchedBranches[0],
          fileExtensions: file_extensions && file_extensions.length > 0 ? file_extensions : prev.fileExtensions
        }));
        
        console.log(`Successfully fetched ${fetchedBranches.length} branches from ${repositoryUrl}`);
        if (file_extensions && file_extensions.length > 0) {
          console.log(`Found ${file_extensions.length} file types: ${file_extensions.join(', ')}`);
        }
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
      setBranches(['main', 'master']); 
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
      case 'github':
      case 'gitlab':
        const isGitHub = provider === 'github';
        return (
          <div className="space-y-6">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <Label htmlFor="accessToken" className="text-sm font-medium">{isGitHub ? 'GitHub' : 'GitLab'} Access Token</Label>
                <a 
                  href={isGitHub 
                    ? "https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token"
                    : "https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html"
                  } 
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
                placeholder={isGitHub 
                  ? "ghp_xxxxxxxxxxxxxxxxxxxx or github_pat_xxxxxxxxxx"
                  : "glpat-xxxxxxxxxxxxxxxxxxxx"
                }
                value={credentials.accessToken || ''}
                onChange={(e: ChangeEvent<HTMLInputElement>) => setCredentials({ ...credentials, accessToken: e.target.value })}
                className="mt-2"
              />
              <div className="space-y-2 text-xs text-muted-foreground">
                <p className="font-medium">Token Requirements:</p>
                <ul className="space-y-1 ml-4">
                  {isGitHub ? (
                    <>
                      <li>• <strong>Classic tokens (ghp_*):</strong> Need <code>repo</code> scope for private repos, <code>public_repo</code> for public repos</li>
                      <li>• <strong>Fine-grained tokens (github_pat_*):</strong> Need repository access and Contents/Metadata permissions</li>
                    </>
                  ) : (
                    <>
                      <li>• <strong>Personal Access Tokens (glpat_*):</strong> Need <code>read_repository</code> scope minimum</li>
                      <li>• <strong>For private repos:</strong> Also need <code>read_api</code> scope</li>
                    </>
                  )}
                  <li>• <strong>Token must not be expired</strong></li>
                  <li>• <strong>Account must have access to the repository</strong></li>
                </ul>
                <div className="flex gap-4 pt-2">
                  {isGitHub ? (
                    <>
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
                    </>
                  ) : (
                    <a 
                      href="https://gitlab.com/-/profile/personal_access_tokens" 
                      target="_blank" 
                      rel="noopener noreferrer"
                      className="text-blue-600 hover:text-blue-800 underline"
                    >
                      Create GitLab Token
                    </a>
                  )}
                </div>
                <div className="mt-3 p-3 bg-muted/50 rounded-md">
                  <p className="font-medium text-foreground">Quick Setup:</p>
                  <ol className="mt-1 space-y-1 ml-4">
                    {isGitHub ? (
                      <>
                        <li>1. Click &quot;Manage Classic Tokens&quot; above</li>
                        <li>2. Generate new token (classic)</li>
                        <li>3. Select <code>public_repo</code> or <code>repo</code> scope</li>
                        <li>4. Copy the token and paste it above</li>
                      </>
                    ) : (
                      <>
                        <li>1. Click &quot;Create GitLab Token&quot; above</li>
                        <li>2. Enter token name and expiration</li>
                        <li>3. Select <code>read_repository</code> and <code>read_api</code> scopes</li>
                        <li>4. Copy the token and paste it above</li>
                      </>
                    )}
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
                onChange={(e: ChangeEvent<HTMLInputElement>) => setCredentials({ ...credentials, username: e.target.value })}
                className="mt-2"
              />
            </div>
            <div className="space-y-3">
              <Label htmlFor="appPassword" className="text-sm font-medium">App Password</Label>
              <Input
                id="appPassword"
                type="password"
                value={credentials.appPassword || ''}
                onChange={(e: ChangeEvent<HTMLInputElement>) => setCredentials({ ...credentials, appPassword: e.target.value })}
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
              onChange={(e: ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
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
                <SelectItem value="gitlab">
                  <div className="flex items-center gap-2">
                    <GitlabIcon className="w-4 h-4" />
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
                  placeholder="https://github.com/user/repo.git"
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
                  <p className="text-sm text-red-700">{fetchBranchesError}</p>
                </div>
              )}
              {branches.length > 0 && !fetchBranchesError && (
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

          {isProviderSelected && (
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
                          {['.js', '.ts', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.md', '.txt', '.json', '.yml', '.yaml'].map((ext) => (
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
              disabled={!isProviderSelected || !repositoryUrl || loading || (!credentials.accessToken && !credentials.username)}
            >
              {loading ? 'Connecting...' : 'Connect Repository'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
