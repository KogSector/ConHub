import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { 
  GitBranch, 
  Plus, 
  Settings, 
  Trash2,
  ExternalLink,
  Eye,
  EyeOff,
  Webhook,
  RefreshCw
} from "lucide-react";

export function RepositorySettings() {
  const repositories = [
    { 
      id: 1, 
      name: "frontend-app", 
      status: "active", 
      private: false, 
      lastSync: "2 minutes ago",
      branch: "main",
      webhookEnabled: true 
    },
    { 
      id: 2, 
      name: "api-gateway", 
      status: "active", 
      private: true, 
      lastSync: "1 hour ago",
      branch: "main",
      webhookEnabled: true 
    },
    { 
      id: 3, 
      name: "user-service", 
      status: "syncing", 
      private: true, 
      lastSync: "Syncing...",
      branch: "develop",
      webhookEnabled: false 
    },
    { 
      id: 4, 
      name: "payment-service", 
      status: "error", 
      private: true, 
      lastSync: "Failed 5 minutes ago",
      branch: "main",
      webhookEnabled: true 
    },
  ];

  return (
    <div className="space-y-6">
      {/* Add Repository */}
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
              <Select defaultValue="main">
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
          </div>
          <div className="flex items-center space-x-2">
            <Switch id="webhook" />
            <Label htmlFor="webhook">Enable webhook for real-time sync</Label>
          </div>
          <Button className="w-full md:w-auto">
            <Plus className="w-4 h-4 mr-2" />
            Connect Repository
          </Button>
        </CardContent>
      </Card>

      {/* Connected Repositories */}
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
            <div key={repo.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-3">
                  <GitBranch className="w-5 h-5 text-primary" />
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-foreground">{repo.name}</span>
                      {repo.private ? (
                        <EyeOff className="w-4 h-4 text-muted-foreground" />
                      ) : (
                        <Eye className="w-4 h-4 text-muted-foreground" />
                      )}
                      <Badge 
                        variant={
                          repo.status === "active" ? "default" : 
                          repo.status === "syncing" ? "secondary" : 
                          "destructive"
                        }
                        className="text-xs"
                      >
                        {repo.status}
                      </Badge>
                    </div>
                    <div className="text-sm text-muted-foreground">
                      Branch: {repo.branch} • Last sync: {repo.lastSync}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
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

              {/* Repository Settings */}
              <div className="grid gap-4 md:grid-cols-3 pt-3 border-t border-border">
                <div className="space-y-2">
                  <Label className="text-sm">Sync Branch</Label>
                  <Select defaultValue={repo.branch}>
                    <SelectTrigger className="h-8">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="main">main</SelectItem>
                      <SelectItem value="master">master</SelectItem>
                      <SelectItem value="develop">develop</SelectItem>
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

      {/* Sync Settings */}
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

      {/* Save Changes */}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
