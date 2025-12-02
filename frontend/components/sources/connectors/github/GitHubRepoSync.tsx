"use client"

import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Progress } from '@/components/ui/progress';
import { useToast } from '@/hooks/use-toast';
import { apiClient, dataApiClient } from '@/lib/api';
import { useAuth } from '@/hooks/use-auth';
import { 
  GitBranch, 
  RefreshCw, 
  FolderGit2, 
  Play,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Settings2,
  Code2,
  FileCode
} from 'lucide-react';
import GitHubIcon from '@/components/icons/GitHubIcon';

interface Repository {
  name: string;
  full_name: string;
  default_branch: string;
}

interface SyncResult {
  success: boolean;
  documents_processed: number;
  embeddings_created: number;
  sync_duration_ms: number;
  error_message?: string;
  graph_job_id?: string;
}

interface GitHubRepoSyncProps {
  onSyncComplete?: (result: SyncResult) => void;
}

const COMMON_LANGUAGES = [
  'rust', 'typescript', 'javascript', 'python', 'go', 'java', 
  'c', 'cpp', 'csharp', 'ruby', 'php', 'swift', 'kotlin'
];

const DEFAULT_EXCLUDE_PATHS = [
  'node_modules', 'dist', 'build', '.git', '__pycache__', 
  'target', 'vendor', '.next', 'coverage'
];

export function GitHubRepoSync({ onSyncComplete }: GitHubRepoSyncProps) {
  const [repos, setRepos] = useState<Repository[]>([]);
  const [branches, setBranches] = useState<string[]>([]);
  const [selectedRepo, setSelectedRepo] = useState<string>('');
  const [selectedBranch, setSelectedBranch] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [loadingRepos, setLoadingRepos] = useState(false);
  const [loadingBranches, setLoadingBranches] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [syncProgress, setSyncProgress] = useState(0);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [includeLanguages, setIncludeLanguages] = useState<string[]>([]);
  const [excludePaths, setExcludePaths] = useState<string[]>(DEFAULT_EXCLUDE_PATHS);
  const [maxFileSizeMb, setMaxFileSizeMb] = useState<number>(5);
  const [lastSyncResult, setLastSyncResult] = useState<SyncResult | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  const { toast } = useToast();
  const { token } = useAuth();

  // Check if GitHub is connected
  const checkConnection = useCallback(async () => {
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      const resp = await apiClient.get<{ success: boolean; data: Array<{ platform: string; is_active: boolean }> }>(
        '/api/auth/connections',
        headers
      );
      const connections = resp.data || [];
      const github = connections.find((c) => c.platform === 'github' && c.is_active);
      setIsConnected(!!github);
    } catch (error) {
      console.error('Failed to check GitHub connection:', error);
      setIsConnected(false);
    }
  }, [token]);

  // Fetch repositories
  const fetchRepos = useCallback(async () => {
    if (!isConnected) return;
    
    setLoadingRepos(true);
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      const resp = await apiClient.get<{ repos: Repository[] }>('/api/auth/repos/github', headers);
      setRepos(resp.repos || []);
    } catch (error) {
      console.error('Failed to fetch repositories:', error);
      toast({
        title: "Error",
        description: "Failed to fetch GitHub repositories",
        variant: "destructive"
      });
    } finally {
      setLoadingRepos(false);
    }
  }, [token, isConnected, toast]);

  // Fetch branches for selected repo
  const fetchBranches = useCallback(async (repoFullName: string) => {
    setLoadingBranches(true);
    setBranches([]);
    setSelectedBranch('');
    
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      const resp = await apiClient.get<{ branches: string[] }>(
        `/api/auth/repos/github/branches?repo=${encodeURIComponent(repoFullName)}`,
        headers
      );
      const branchList = resp.branches || [];
      setBranches(branchList);
      
      // Auto-select default branch if available
      const selectedRepoData = repos.find(r => r.full_name === repoFullName);
      if (selectedRepoData && branchList.includes(selectedRepoData.default_branch)) {
        setSelectedBranch(selectedRepoData.default_branch);
      } else if (branchList.length > 0) {
        setSelectedBranch(branchList[0]);
      }
    } catch (error) {
      console.error('Failed to fetch branches:', error);
      toast({
        title: "Error",
        description: "Failed to fetch repository branches",
        variant: "destructive"
      });
    } finally {
      setLoadingBranches(false);
    }
  }, [token, repos, toast]);

  // Handle repo selection
  const handleRepoSelect = (repoFullName: string) => {
    setSelectedRepo(repoFullName);
    fetchBranches(repoFullName);
  };

  // Sync repository
  const syncRepository = async () => {
    if (!selectedRepo || !selectedBranch) {
      toast({
        title: "Error",
        description: "Please select a repository and branch",
        variant: "destructive"
      });
      return;
    }

    setSyncing(true);
    setSyncProgress(10);
    setLastSyncResult(null);

    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      
      // Simulate progress
      const progressInterval = setInterval(() => {
        setSyncProgress(prev => Math.min(prev + 10, 90));
      }, 1000);

      const result = await dataApiClient.post<SyncResult>(
        '/api/github/sync',
        {
          repo_url: `https://github.com/${selectedRepo}`,
          branch: selectedBranch,
          include_languages: includeLanguages.length > 0 ? includeLanguages : null,
          exclude_paths: excludePaths.length > 0 ? excludePaths : null,
          max_file_size_mb: maxFileSizeMb,
        },
        headers
      );

      clearInterval(progressInterval);
      setSyncProgress(100);
      setLastSyncResult(result);

      if (result.success) {
        toast({
          title: "Sync Complete",
          description: `Processed ${result.documents_processed} files in ${(result.sync_duration_ms / 1000).toFixed(1)}s`,
        });
        onSyncComplete?.(result);
      } else {
        toast({
          title: "Sync Failed",
          description: result.error_message || "Unknown error occurred",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Sync failed:', error);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setLastSyncResult({
        success: false,
        documents_processed: 0,
        embeddings_created: 0,
        sync_duration_ms: 0,
        error_message: errorMessage
      });
      toast({
        title: "Sync Failed",
        description: errorMessage,
        variant: "destructive"
      });
    } finally {
      setSyncing(false);
      setTimeout(() => setSyncProgress(0), 2000);
    }
  };

  // Toggle language filter
  const toggleLanguage = (lang: string) => {
    setIncludeLanguages(prev => 
      prev.includes(lang) 
        ? prev.filter(l => l !== lang)
        : [...prev, lang]
    );
  };

  useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  useEffect(() => {
    if (isConnected) {
      fetchRepos();
    }
  }, [isConnected, fetchRepos]);

  if (!isConnected) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <GitHubIcon className="h-5 w-5" />
            GitHub Repository Sync
          </CardTitle>
          <CardDescription>
            Connect your GitHub account to sync repositories for AI-powered code search
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <div className="text-center space-y-4">
              <AlertCircle className="h-12 w-12 mx-auto text-muted-foreground" />
              <p className="text-muted-foreground">
                GitHub account not connected
              </p>
              <p className="text-sm text-muted-foreground">
                Go to <strong>Connections</strong> to connect your GitHub account first.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <GitHubIcon className="h-5 w-5" />
          GitHub Repository Sync
        </CardTitle>
        <CardDescription>
          Sync your repositories to enable AI-powered code search and understanding
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Repository Selection */}
        <div className="space-y-2">
          <Label>Repository</Label>
          <div className="flex gap-2">
            <Select value={selectedRepo} onValueChange={handleRepoSelect} disabled={loadingRepos}>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={loadingRepos ? "Loading repositories..." : "Select a repository"} />
              </SelectTrigger>
              <SelectContent>
                {repos.map((repo) => (
                  <SelectItem key={repo.full_name} value={repo.full_name}>
                    <div className="flex items-center gap-2">
                      <FolderGit2 className="h-4 w-4" />
                      {repo.full_name}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button 
              variant="outline" 
              size="icon"
              onClick={fetchRepos}
              disabled={loadingRepos}
            >
              <RefreshCw className={`h-4 w-4 ${loadingRepos ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </div>

        {/* Branch Selection */}
        {selectedRepo && (
          <div className="space-y-2">
            <Label>Branch</Label>
            <Select 
              value={selectedBranch} 
              onValueChange={setSelectedBranch} 
              disabled={loadingBranches}
            >
              <SelectTrigger>
                <SelectValue placeholder={loadingBranches ? "Loading branches..." : "Select a branch"} />
              </SelectTrigger>
              <SelectContent>
                {branches.map((branch) => (
                  <SelectItem key={branch} value={branch}>
                    <div className="flex items-center gap-2">
                      <GitBranch className="h-4 w-4" />
                      {branch}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {/* Advanced Settings Toggle */}
        <Button 
          variant="ghost" 
          size="sm" 
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="flex items-center gap-2"
        >
          <Settings2 className="h-4 w-4" />
          {showAdvanced ? 'Hide' : 'Show'} Advanced Settings
        </Button>

        {/* Advanced Settings */}
        {showAdvanced && (
          <div className="space-y-4 p-4 border rounded-lg bg-muted/50">
            {/* Language Filters */}
            <div className="space-y-2">
              <Label className="flex items-center gap-2">
                <Code2 className="h-4 w-4" />
                Include Languages (empty = all)
              </Label>
              <div className="flex flex-wrap gap-2">
                {COMMON_LANGUAGES.map((lang) => (
                  <Badge
                    key={lang}
                    variant={includeLanguages.includes(lang) ? "default" : "outline"}
                    className="cursor-pointer"
                    onClick={() => toggleLanguage(lang)}
                  >
                    {lang}
                  </Badge>
                ))}
              </div>
            </div>

            {/* Exclude Paths */}
            <div className="space-y-2">
              <Label className="flex items-center gap-2">
                <FileCode className="h-4 w-4" />
                Exclude Paths
              </Label>
              <Input
                value={excludePaths.join(', ')}
                onChange={(e) => setExcludePaths(e.target.value.split(',').map(p => p.trim()).filter(Boolean))}
                placeholder="node_modules, dist, .git"
              />
              <p className="text-xs text-muted-foreground">Comma-separated list of paths to exclude</p>
            </div>

            {/* Max File Size */}
            <div className="space-y-2">
              <Label>Max File Size (MB)</Label>
              <Input
                type="number"
                value={maxFileSizeMb}
                onChange={(e) => setMaxFileSizeMb(parseInt(e.target.value) || 5)}
                min={1}
                max={50}
              />
            </div>
          </div>
        )}

        {/* Sync Progress */}
        {syncing && (
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>Syncing repository...</span>
              <span>{syncProgress}%</span>
            </div>
            <Progress value={syncProgress} />
          </div>
        )}

        {/* Last Sync Result */}
        {lastSyncResult && !syncing && (
          <div className={`p-4 rounded-lg ${lastSyncResult.success ? 'bg-green-500/10 border border-green-500/20' : 'bg-red-500/10 border border-red-500/20'}`}>
            <div className="flex items-center gap-2 mb-2">
              {lastSyncResult.success ? (
                <CheckCircle2 className="h-5 w-5 text-green-500" />
              ) : (
                <AlertCircle className="h-5 w-5 text-red-500" />
              )}
              <span className="font-medium">
                {lastSyncResult.success ? 'Sync Successful' : 'Sync Failed'}
              </span>
            </div>
            {lastSyncResult.success ? (
              <div className="text-sm text-muted-foreground space-y-1">
                <p>📄 {lastSyncResult.documents_processed} files processed</p>
                <p>🔢 {lastSyncResult.embeddings_created} embeddings created</p>
                <p>⏱️ {(lastSyncResult.sync_duration_ms / 1000).toFixed(1)}s duration</p>
                {lastSyncResult.graph_job_id && (
                  <p>📊 Graph ingestion job: {lastSyncResult.graph_job_id.slice(0, 8)}...</p>
                )}
              </div>
            ) : (
              <p className="text-sm text-red-500">{lastSyncResult.error_message}</p>
            )}
          </div>
        )}

        {/* Sync Button */}
        <Button 
          onClick={syncRepository}
          disabled={!selectedRepo || !selectedBranch || syncing}
          className="w-full"
          size="lg"
        >
          {syncing ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Syncing...
            </>
          ) : (
            <>
              <Play className="mr-2 h-4 w-4" />
              Sync Repository
            </>
          )}
        </Button>

        <p className="text-xs text-muted-foreground text-center">
          This will fetch all code files and index them for AI-powered search via both vector and graph RAG.
        </p>
      </CardContent>
    </Card>
  );
}

export default GitHubRepoSync;
