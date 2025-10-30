'use client';

import { useToast } from "@/hooks/use-toast";
import { useMemo, useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { apiClient, unwrapResponse } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { GitBranch, Github, GitlabIcon, GitlabIcon as Bitbucket, AlertCircle } from 'lucide-react';

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
    
    if (!repositoryUrl || !provider) {
      setFetchBranchesError("Please enter a repository URL and select a provider first.");
      return;
    }
    
    
    if (!isUrlValid) {
      setFetchBranchesError("Please enter a valid repository URL.");
      return;
    }
    
    
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
      
      let credentialsPayload;
      if (provider === 'github' || provider === 'gitlab') {
        credentialsPayload = {
          credential_type: {
            PersonalAccessToken: {
              token: credentials.accessToken.trim()
            }
          },
          expires_at: null
        };
      } else if (provider === 'bitbucket') {
        credentialsPayload = {
          credential_type: {
            AppPassword: {
              username: credentials.username.trim(),
              app_password: credentials.appPassword.trim()
            }
          },
          expires_at: null
        };
      }

      
      const urlResp = await apiClient.post('/api/repositories/validate-url', { repo_url: repositoryUrl.trim() });
      const urlValidation = unwrapResponse<{ success?: boolean; error?: string }>(urlResp) ?? urlResp;
      if (!urlValidation || !urlValidation.success) {
        const errMsg = urlValidation && typeof (urlValidation as Record<string, unknown>)['error'] === 'string'
          ? (urlValidation as Record<string, unknown>)['error'] as string
          : 'Invalid repository URL format.';
        throw new Error(errMsg);
      }

      
      const resp = await apiClient.post('/api/repositories/fetch-branches', { repo_url: repositoryUrl.trim(), credentials: credentialsPayload });
      const data = unwrapResponse<{ success?: boolean; data?: { branches: string[]; default_branch?: string }; error?: string }>(resp) ?? resp;
      if (!data || !data.success) {
        const errMsg = data && typeof (data as Record<string, unknown>)['error'] === 'string'
          ? (data as Record<string, unknown>)['error'] as string
          : 'Failed to fetch branches.';
        throw new Error(errMsg);
      }
      const { branches: fetchedBranches, default_branch } = data.data || { branches: [], default_branch: undefined };
      
      if (!fetchedBranches || fetchedBranches.length === 0) {
        setFetchBranchesError("No branches found. Please check the repository URL and permissions.");
        setBranches(['main', 'master']); 
        setConfig(prev => ({ ...prev, defaultBranch: 'main' }));
      } else {
        setBranches(fetchedBranches);
        setConfig(prev => ({ ...prev, defaultBranch: default_branch || fetchedBranches[0] }));
        
        console.log(`Successfully fetched ${fetchedBranches.length} branches from ${repositoryUrl}`);
      }
    } catch (err: unknown) {
      console.error('Branch fetching error:', err);
      const errorMessage = err instanceof Error ? err.message : String(err);
      
      
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
      
      const resp = await apiClient.post('/api/data-sources/connect', payload);
      const data = unwrapResponse<{ success?: boolean; error?: string }>(resp) ?? resp;
      if (data && data.success) {
        onSuccess();
        onOpenChange(false);
        resetForm();
      } else {
        const errorMessage = (data && (data as Record<string, unknown>)['error']) as string || 'Failed to connect repository';
        
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
                onChange={(e) => setCredentials({ ...credentials, accessToken: e.target.value })}
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
                        <li>1. Click "Manage Classic Tokens" above</li>
                        <li>2. Generate new token (classic)</li>
                        <li>3. Select <code>public_repo</code> or <code>repo</code> scope</li>
                        <li>4. Copy the token and paste it above</li>
                      </>
                    ) : (
                      <>
                        <li>1. Click "Create GitLab Token" above</li>
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
                <SelectItem value="gitlab">
                  <div className="flex items-center gap-2">
                    <GitlabIcon className="w-4 h-4" />
                    GitLab
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
                  className="bg-blue-600 hover:bg-blue-700 text-white"
                  onClick={handleFetchBranches}
                  disabled={isFetchingBranches || !isUrlValid || !provider}
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
                </div>
              )}
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