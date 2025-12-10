'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, GitBranch, Plus, Settings, ExternalLink, Star, GitFork, RefreshCw, MoreHorizontal, Trash2, GitPullRequest } from "lucide-react";
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
import { ConnectRepositoryDialog } from "./ConnectRepositoryDialog";
import { dataApiClient, ApiResponse } from '@/lib/api';
import { ChangeBranchDialog } from "./ChangeBranchDialog";
import { useAuth } from '@/hooks/use-auth';

interface Repository {
  id: string;
  name: string;
  description: string;
  language: string;
  stars: number;
  forks: number;
  lastUpdated: string;
  status: 'active' | 'inactive' | 'syncing' | 'error';
  url: string;
  provider: 'github' | 'bitbucket';
  defaultBranch?: string;
  // Sync config options
  fileExtensions?: string[];
  fetchIssues?: boolean;
  fetchPrs?: boolean;
  autoSync?: boolean;
}

interface SyncResult {
  success: boolean;
  documents_processed: number;
  embeddings_created: number;
  sync_duration_ms: number;
  error_message?: string;
  graph_job_id?: string;
  issues_processed?: number;
  prs_processed?: number;
}

interface DataSource {
  id: string;
  type: string;
  name: string;
  status: 'connected' | 'syncing' | 'error' | 'indexing';
  totalCount?: number;
  indexedCount?: number;
  config?: any;
}

export function RepositoriesPageClient() {
  const { token } = useAuth();
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [dataSources, setDataSources] = useState<DataSource[]>([]);
  const [showConnectDialog, setShowConnectDialog] = useState(false);
  const [showChangeBranchDialog, setShowChangeBranchDialog] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [selectedRepoId, setSelectedRepoId] = useState<string | null>(null);
  const [currentBranch, setCurrentBranch] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncingRepoId, setSyncingRepoId] = useState<string | null>(null);
  const [syncStatus, setSyncStatus] = useState<{ [key: string]: { status: string; message?: string } }>({});

  
  const sampleRepositories: Repository[] = [
    {
      id: "1",
      name: "ConHub",
      description: "A unified platform for connecting repositories, documents, and AI agents",
      language: "TypeScript",
      stars: 42,
      forks: 8,
      lastUpdated: "2 hours ago",
      status: "active",
      url: "https://github.com/KogSector/ConHub",
      provider: "github"
    },
    {
      id: "2",
      name: "data-processor",
      description: "High-performance data processing pipeline for ML workflows",
      language: "Python",
      stars: 127,
      forks: 23,
      lastUpdated: "1 day ago",
      status: "active",
      url: "https://github.com/KogSector/data-processor",
      provider: "github"
    },
    {
      id: "3",
      name: "api-gateway",
      description: "Microservices API gateway with authentication and rate limiting",
      language: "Go",
      stars: 89,
      forks: 15,
      lastUpdated: "3 days ago",
      status: "inactive",
      url: "https://github.com/KogSector/api-gateway",
      provider: "github"
    }
  ];

  useEffect(() => {
    fetchRepositories();
    fetchDataSources();
  }, []);

  const fetchRepositories = async () => {
    try {
      // Fetch real repository data from the API with auth header
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      const resp = await dataApiClient.get<ApiResponse<{ repositories: Repository[] }>>('/api/repositories', headers);
      if (resp.success && resp.data?.repositories) {
        setRepositories(resp.data.repositories);
      } else {
        // If no repositories found, show empty state
        setRepositories([]);
      }
    } catch (error) {
      console.error('Error fetching repositories:', error);
      // Fallback to empty array on error
      setRepositories([]);
    } finally {
      setLoading(false);
    }
  };

  const fetchDataSources = async () => {
    try {
        // Fetch data sources with auth header
        const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
        const resp = await dataApiClient.get<ApiResponse<{ dataSources: DataSource[] }>>('/api/data-sources', headers);
        if (resp.success && resp.data) {
          const repoDataSources = resp.data.dataSources?.filter((ds: DataSource) => 
            ['github', 'bitbucket'].includes(ds.type)
          ) || [];
          setDataSources(repoDataSources);
        }
    } catch (error) {
      console.error('Error fetching data sources:', error);
      
      setDataSources([]);
    }
  };

  const handleRepositoryConnected = () => {
    fetchRepositories();
    fetchDataSources();
  };

  const openChangeBranchDialog = (repoId: string, branch: string) => {
    setSelectedRepoId(repoId);
    setCurrentBranch(branch);
    setShowChangeBranchDialog(true);
  };

  const deleteRepository = async () => {
    if (!selectedRepoId) return;
    try {
      
      console.log(`Deleting repository ${selectedRepoId}`);
      await new Promise(resolve => setTimeout(resolve, 1000)); 
      fetchRepositories();
      fetchDataSources();
    } catch (error) {
      console.error('Error deleting repository:', error);
    } finally {
      setShowDeleteConfirm(false);
      setSelectedRepoId(null);
    }
  };

  const syncRepository = async (repo: Repository) => {
    if (!repo.url || syncingRepoId) return;
    
    setSyncingRepoId(repo.id);
    setSyncStatus(prev => ({ ...prev, [repo.id]: { status: 'syncing', message: 'Starting sync...' } }));
    
    try {
      const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};
      
      // Call the GitHub sync endpoint with config options
      const resp = await dataApiClient.post<SyncResult>('/api/github/sync', {
        repo_url: repo.url,
        branch: repo.defaultBranch || 'main',
        include_languages: null, // All languages
        exclude_paths: ['node_modules', 'dist', 'build', '.git', 'target', '__pycache__'],
        max_file_size_mb: 5,
        // Pass file extensions if configured
        file_extensions: repo.fileExtensions || null,
        // Pass issues/PRs flags (default to true for backwards compatibility)
        fetch_issues: repo.fetchIssues ?? true,
        fetch_prs: repo.fetchPrs ?? true,
      }, headers);
      
      if (resp.success) {
        setSyncStatus(prev => ({
          ...prev,
          [repo.id]: {
            status: 'success',
            message: `Synced ${resp.documents_processed} docs${resp.issues_processed ? `, ${resp.issues_processed} issues` : ''}${resp.prs_processed ? `, ${resp.prs_processed} PRs` : ''} in ${(resp.sync_duration_ms / 1000).toFixed(1)}s`
          }
        }));
        
        // Update repository status
        setRepositories(prev => prev.map(r => 
          r.id === repo.id ? { ...r, status: 'active' as const } : r
        ));
        
        fetchDataSources();
      } else {
        setSyncStatus(prev => ({
          ...prev,
          [repo.id]: {
            status: 'error',
            message: resp.error_message || 'Sync failed'
          }
        }));
      }
    } catch (error) {
      console.error('Error syncing repository:', error);
      setSyncStatus(prev => ({
        ...prev,
        [repo.id]: {
          status: 'error',
          message: error instanceof Error ? error.message : 'Sync failed'
        }
      }));
    } finally {
      setSyncingRepoId(null);
      // Clear status after 5 seconds
      setTimeout(() => {
        setSyncStatus(prev => {
          const { [repo.id]: _, ...rest } = prev;
          return rest;
        });
      }, 5000);
    }
  };

  const languageColors = {
    TypeScript: "bg-blue-500",
    Python: "bg-yellow-500",
    Go: "bg-cyan-500",
    JavaScript: "bg-yellow-400",
    Rust: "bg-orange-600"
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-500 shadow-lg shadow-green-500/50';
      case 'syncing': return 'bg-blue-500 shadow-lg shadow-blue-500/50';
      case 'error': return 'bg-red-500 shadow-lg shadow-red-500/50';
      default: return 'bg-gray-400';
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Loading repositories...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard">
                <Button variant="ghost" size="sm">
                  <ArrowLeft className="w-4 h-4" />
                </Button>
              </Link>
              <div className="flex items-center space-x-3">
                <GitBranch className="w-6 h-6 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">Repositories</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Repositories
              </CardTitle>
              <GitBranch className="w-4 h-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">{repositories.length}</div>
              <p className="text-xs text-muted-foreground">
                {dataSources.length} connections
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Active Repos
              </CardTitle>
              <GitBranch className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {repositories.filter(r => r.status === 'active').length}
              </div>
              <p className="text-xs text-muted-foreground">
                Currently syncing
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Stars
              </CardTitle>
              <Star className="w-4 h-4 text-yellow-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {repositories.reduce((sum, repo) => sum + repo.stars, 0)}
              </div>
              <p className="text-xs text-muted-foreground">
                Community recognition
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Forks
              </CardTitle>
              <GitFork className="w-4 h-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {repositories.reduce((sum, repo) => sum + repo.forks, 0)}
              </div>
              <p className="text-xs text-muted-foreground">
                Community contributions
              </p>
            </CardContent>
          </Card>
        </div>

        {}
        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-xl font-semibold text-foreground">Connected Repositories</h2>
            <p className="text-sm text-muted-foreground">Connect and manage your code repositories</p>
          </div>
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => fetchDataSources()}>
              <RefreshCw className="w-4 h-4 mr-2" />
              Refresh
            </Button>
            <Button onClick={() => setShowConnectDialog(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Connect Repository
            </Button>
          </div>
        </div>

        {}
        <div className="space-y-4">
          {repositories.length === 0 ? (
            <Card className="bg-muted/50 border-dashed border-muted-foreground/25">
              <CardContent className="flex flex-col items-center justify-center py-12">
                <GitBranch className="w-12 h-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-medium text-foreground mb-2">No repositories connected</h3>
                <p className="text-muted-foreground text-center mb-6 max-w-md">
                  Connect your first repository to start indexing code and enable AI-powered development workflows.
                </p>
                <Button onClick={() => setShowConnectDialog(true)}>
                  <Plus className="w-4 h-4 mr-2" />
                  Connect Your First Repository
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4">
              {repositories.map((repo) => (
                <Card key={repo.id} className="bg-card border-border hover:bg-accent/5 transition-colors overflow-hidden">
                  <div className="flex flex-col px-6 py-4 gap-4">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="font-semibold text-base text-foreground truncate min-w-0 flex-1">{repo.name}</span>
                      <Badge variant="outline" className="flex-shrink-0">
                        {repo.provider}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-2 min-w-0">
                      <div className={`w-2 h-2 rounded-full flex-shrink-0 ${getStatusColor(repo.status)}`}></div>
                      <p className="text-xs text-muted-foreground truncate min-w-0 flex-1">{repo.description}</p>
                    </div>
                    <div className="flex items-center gap-4 text-xs text-muted-foreground flex-wrap">
                      <div className="flex items-center gap-1 flex-shrink-0">
                        <div className={`w-2 h-2 rounded-full ${languageColors[repo.language as keyof typeof languageColors] || 'bg-gray-500'}`}></div>
                        <span className="truncate">{repo.language}</span>
                      </div>
                      <div className="flex items-center gap-1 flex-shrink-0">
                        <span>{repo.stars}</span>
                      </div>
                      <div className="flex items-center gap-1 flex-shrink-0">
                        <span>{repo.forks}</span>
                      </div>
                      <span className="truncate">Updated {repo.lastUpdated}</span>
                    </div>
                    <div className="flex items-center gap-2 flex-wrap">
                      <Button variant="outline" size="sm" asChild>
                        <Link href={repo.url} target="_blank" rel="noopener noreferrer">
                          <ExternalLink className="w-4 h-4 mr-1" />
                          View on {repo.provider === 'github' ? 'GitHub' : 'BitBucket'}
                        </Link>
                      </Button>
                      <Button 
                        variant="outline" 
                        size="sm"
                        onClick={() => syncRepository(repo)}
                        disabled={syncingRepoId === repo.id || repo.status === 'syncing'}
                      >
                        <RefreshCw className={`w-4 h-4 mr-1 ${syncingRepoId === repo.id ? 'animate-spin' : ''}`} />
                        {syncingRepoId === repo.id ? 'Syncing...' : syncStatus[repo.id]?.status === 'success' ? '✓ Synced' : syncStatus[repo.id]?.status === 'error' ? '✗ Error' : 'Sync'}
                      </Button>
                      {syncStatus[repo.id]?.message && (
                        <span className={`text-xs ${syncStatus[repo.id]?.status === 'error' ? 'text-red-500' : 'text-green-500'}`}>
                          {syncStatus[repo.id]?.message}
                        </span>
                      )}
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="outline" size="sm">
                            <MoreHorizontal className="w-4 h-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => openChangeBranchDialog(repo.id, 'main')}>
                            <GitPullRequest className="w-4 h-4 mr-2" />
                            Change Branch
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            className="text-red-500"
                            onClick={() => {
                              setSelectedRepoId(repo.id);
                              setShowDeleteConfirm(true);
                            }}
                          >
                            <Trash2 className="w-4 h-4 mr-2" />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>
      
      <Footer />

      <ConnectRepositoryDialog
        open={showConnectDialog}
        onOpenChange={setShowConnectDialog}
        onSuccess={handleRepositoryConnected}
      />
      <ChangeBranchDialog
        open={showChangeBranchDialog}
        onOpenChange={setShowChangeBranchDialog}
        repositoryId={selectedRepoId}
        currentBranch={currentBranch}
        onSuccess={() => {
          fetchRepositories();
          fetchDataSources();
        }}
      />
      <AlertDialog open={showDeleteConfirm} onOpenChange={setShowDeleteConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Are you sure?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. This will permanently delete the
              repository and all associated data.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={deleteRepository}>
              Continue
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
