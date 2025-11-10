import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useState, useEffect } from "react";
import { apiClient, ApiResponse } from '@/lib/api';
import { 
  GitBranch, 
  Plus, 
  Settings, 
  Trash2,
  ExternalLink,
  Eye,
  EyeOff,
  Webhook,
  RefreshCw,
  ChevronRight,
  ChevronDown
} from "lucide-react";

export function RepositorySettings() {
  const [repositories, setRepositories] = useState([
    { 
      id: 1, 
      name: "frontend-app", 
      status: "active", 
      private: false, 
      lastSync: "2 minutes ago",
      branch: "main",
      webhookEnabled: true,
      branches: [] as string[]
    },
    { 
      id: 2, 
      name: "api-gateway", 
      status: "active", 
      private: true, 
      lastSync: "1 hour ago",
      branch: "main",
      webhookEnabled: true,
      branches: [] as string[]
    },
    { 
      id: 3, 
      name: "user-service", 
      status: "syncing", 
      private: true, 
      lastSync: "Syncing...",
      branch: "develop",
      webhookEnabled: false,
      branches: [] as string[]
    },
    { 
      id: 4, 
      name: "payment-service", 
      status: "error", 
      private: true, 
      lastSync: "Failed 5 minutes ago",
      branch: "main",
      webhookEnabled: true,
      branches: [] as string[]
    },
  ]);
  const [showAdvancedSettings, setShowAdvancedSettings] = useState(false);
  const [loadingBranches, setLoadingBranches] = useState<Record<number, boolean>>({});

  const fetchBranches = async (repoId: number) => {
    setLoadingBranches(prev => ({ ...prev, [repoId]: true }));
    try {
      const resp = await apiClient.get<ApiResponse<{ branches: string[]; defaultBranch: string }>>(
        `/api/repositories/${repoId}/branches`
      );
      
      if (resp.success && resp.data) {
        setRepositories(prev => prev.map(repo => 
          repo.id === repoId 
            ? { ...repo, branches: resp.data!.branches || ['main', 'master', 'develop'] }
            : repo
        ));
      }
    } catch (error) {
      console.error('Error fetching branches:', error);
      // Set fallback branches
      setRepositories(prev => prev.map(repo => 
        repo.id === repoId 
          ? { ...repo, branches: ['main', 'master', 'develop'] }
          : repo
      ));
    } finally {
      setLoadingBranches(prev => ({ ...prev, [repoId]: false }));
    }
  };

  useEffect(() => {
    // Fetch branches for all connected repositories on component mount
    repositories.forEach(repo => {
      if (repo.status === 'active') {
        fetchBranches(repo.id);
      }
    });
  }, []);

  return (
    <div className="space-y-6">
      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Plus className="w-5 h-5" />
            Connect New Repository
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="repoUrl">Repository URL</Label>
              <Input 
                id="repoUrl" 
                placeholder="https://github.com/username/repository" 
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="branch">Default Branch</Label>
              <Select defaultValue="main" disabled>
                <SelectTrigger className="opacity-50 cursor-not-allowed">
                  <SelectValue placeholder="Connect repository first" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="" disabled>
                    No repository connected
                  </SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                Branch selection will be enabled after connecting a repository
              </p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Switch id="webhook" />
            <Label htmlFor="webhook">Enable webhook for real-time sync</Label>
          </div>
          <div className="space-y-4">
            <button
              type="button"
              onClick={() => setShowAdvancedSettings(!showAdvancedSettings)}
              className="flex items-center gap-2 text-sm font-medium hover:text-primary transition-colors"
            >
              {showAdvancedSettings ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
              Advanced Settings
            </button>
            
            <div className={`transition-all duration-300 ease-in-out overflow-hidden ${
              showAdvancedSettings ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'
            }`}>
              <div className="space-y-4 pt-2">
                <div className="space-y-3">
                  <Label className="text-sm font-medium">File Extensions to Index</Label>
                  <div className="flex flex-wrap gap-2">
                    {['.js', '.ts', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.md', '.txt', '.json', '.yml', '.yaml'].map((ext) => (
                      <Badge
                        key={ext}
                        variant="outline"
                        className="cursor-pointer text-xs"
                      >
                        {ext}
                      </Badge>
                    ))}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Configure file types to include in repository indexing
                  </p>
                </div>
              </div>
            </div>
            
            <Button className="w-full md:w-auto">
              <Plus className="w-4 h-4 mr-2" />
              Connect Repository
            </Button>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <GitBranch className="w-5 h-5" />
              Connected Repositories ({repositories.length})
            </CardTitle>
            <Button variant="outline" size="sm">
              <RefreshCw className="w-4 h-4 mr-2" />
              Sync All
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {repositories.map((repo) => (
            <div key={repo.id} className="p-4 rounded-lg border border-border bg-muted/20 overflow-hidden">
              <div className="flex items-center justify-between mb-3 min-w-0">
                <div className="flex items-center gap-3 min-w-0 flex-1">
                  <GitBranch className="w-5 h-5 text-primary flex-shrink-0" />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="font-medium text-foreground truncate min-w-0">{repo.name}</span>
                      {repo.private ? (
                        <EyeOff className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                      ) : (
                        <Eye className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                      )}
                      <Badge 
                        variant={
                          repo.status === "active" ? "default" : 
                          repo.status === "syncing" ? "secondary" : 
                          "destructive"
                        }
                        className="text-xs flex-shrink-0"
                      >
                        {repo.status}
                      </Badge>
                    </div>
                    <div className="text-sm text-muted-foreground truncate">
                      Branch: {repo.branch} • Last sync: {repo.lastSync}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  <Button variant="ghost" size="sm">
                    <ExternalLink className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm" className="text-destructive">
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>

              {}
              <div className="grid gap-4 md:grid-cols-3 pt-3 border-t border-border">
                <div className="space-y-2">
                  <Label className="text-sm">Sync Branch</Label>
                  <Select 
                    defaultValue={repo.branch}
                    disabled={repo.status !== 'active' || repo.branches.length === 0}
                    onOpenChange={(open) => {
                      if (open && repo.branches.length === 0 && repo.status === 'active') {
                        fetchBranches(repo.id);
                      }
                    }}
                  >
                    <SelectTrigger className={`h-8 ${repo.status !== 'active' || repo.branches.length === 0 ? 'opacity-50 cursor-not-allowed' : ''}`}>
                      <SelectValue placeholder={loadingBranches[repo.id] ? 'Loading...' : repo.status !== 'active' ? 'Repository inactive' : 'Select branch'} />
                    </SelectTrigger>
                    <SelectContent>
                      {repo.branches.length > 0 ? (
                        repo.branches.map((branch) => (
                          <SelectItem key={branch} value={branch}>
                            {branch}
                          </SelectItem>
                        ))
                      ) : (
                        <SelectItem value="" disabled>
                          {repo.status !== 'active' ? 'Repository not connected' : 'No branches available'}
                        </SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">Webhook</Label>
                  <div className="flex items-center space-x-2">
                    <Switch 
                      id={`webhook-${repo.id}`} 
                      defaultChecked={repo.webhookEnabled} 
                    />
                    <Webhook className="w-4 h-4 text-muted-foreground" />
                  </div>
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">Auto Sync</Label>
                  <div className="flex items-center space-x-2">
                    <Switch 
                      id={`autosync-${repo.id}`} 
                      defaultChecked={true} 
                    />
                    <RefreshCw className="w-4 h-4 text-muted-foreground" />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle>Global Sync Settings</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Auto-sync every</Label>
              <p className="text-sm text-muted-foreground">
                Automatically sync repositories at regular intervals
              </p>
            </div>
            <Select defaultValue="5">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">1 minute</SelectItem>
                <SelectItem value="5">5 minutes</SelectItem>
                <SelectItem value="15">15 minutes</SelectItem>
                <SelectItem value="30">30 minutes</SelectItem>
                <SelectItem value="60">1 hour</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Real-time webhooks</Label>
              <p className="text-sm text-muted-foreground">
                Enable instant sync when changes are pushed to repositories
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Sync on startup</Label>
              <p className="text-sm text-muted-foreground">
                Automatically sync all repositories when ConHub starts
              </p>
            </div>
            <Switch defaultChecked />
          </div>
        </CardContent>
      </Card>

      {}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
