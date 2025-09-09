import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { 
  Code, 
  Key, 
  Webhook,
  Copy,
  Plus,
  Trash2,
  Eye,
  EyeOff,
  Settings,
  Zap,
  Globe,
  Database,
  RefreshCw
} from "lucide-react";

export function DeveloperSettings() {
  const apiKeys = [
    {
      id: 1,
      name: "Production API",
      key: "pk_prod_***********1234",
      created: "2024-08-15",
      lastUsed: "2 minutes ago",
      permissions: ["read", "write", "admin"]
    },
    {
      id: 2,
      name: "Development API",
      key: "pk_dev_***********5678",
      created: "2024-07-20",
      lastUsed: "1 hour ago",
      permissions: ["read", "write"]
    },
    {
      id: 3,
      name: "Testing API",
      key: "pk_test_***********9012",
      created: "2024-06-10",
      lastUsed: "Never",
      permissions: ["read"]
    }
  ];

  const webhooks = [
    {
      id: 1,
      name: "Repository Sync",
      url: "https://api.yourapp.com/webhooks/repo-sync",
      events: ["repository.sync", "repository.error"],
      status: "active",
      lastDelivery: "2 minutes ago"
    },
    {
      id: 2,
      name: "AI Agent Events",
      url: "https://api.yourapp.com/webhooks/ai-events",
      events: ["ai.request", "ai.response", "ai.error"],
      status: "active",
      lastDelivery: "5 minutes ago"
    },
    {
      id: 3,
      name: "User Actions",
      url: "https://api.yourapp.com/webhooks/user-actions",
      events: ["user.login", "user.logout"],
      status: "inactive",
      lastDelivery: "Never"
    }
  ];

  const featureFlags = [
    {
      name: "advanced_ai_features",
      description: "Enable experimental AI features and models",
      enabled: true
    },
    {
      name: "beta_ui_components",
      description: "Show beta UI components and layouts",
      enabled: false
    },
    {
      name: "enhanced_security",
      description: "Enable additional security measures",
      enabled: true
    },
    {
      name: "custom_integrations",
      description: "Allow custom third-party integrations",
      enabled: false
    }
  ];

  return (
    <div className="space-y-6">
      {/* API Keys */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Key className="w-5 h-5" />
              API Keys
            </CardTitle>
            <Button size="sm">
              <Plus className="w-4 h-4 mr-2" />
              Generate New Key
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {apiKeys.map((apiKey) => (
            <div key={apiKey.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between mb-3">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{apiKey.name}</span>
                    <div className="flex gap-1">
                      {apiKey.permissions.map((perm) => (
                        <Badge key={perm} variant="outline" className="text-xs">
                          {perm}
                        </Badge>
                      ))}
                    </div>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Created: {apiKey.created} • Last used: {apiKey.lastUsed}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="ghost" size="sm">
                    <Copy className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm" className="text-destructive">
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Input 
                  type="password" 
                  value={apiKey.key} 
                  readOnly 
                  className="h-8 text-xs font-mono"
                />
                <Button variant="outline" size="sm">
                  <Eye className="w-4 h-4" />
                </Button>
              </div>
            </div>
          ))}

          {/* Generate New API Key Form */}
          <div className="p-4 rounded-lg border-2 border-dashed border-border">
            <h4 className="font-medium mb-3">Generate New API Key</h4>
            <div className="grid gap-3 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="keyName">Key Name</Label>
                <Input id="keyName" placeholder="My API Key" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="keyPermissions">Permissions</Label>
                <Select>
                  <SelectTrigger>
                    <SelectValue placeholder="Select permissions" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="read">Read only</SelectItem>
                    <SelectItem value="read-write">Read & Write</SelectItem>
                    <SelectItem value="admin">Full admin access</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <Button className="mt-3">
              <Key className="w-4 h-4 mr-2" />
              Generate Key
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Webhooks */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Webhook className="w-5 h-5" />
              Webhooks
            </CardTitle>
            <Button size="sm">
              <Plus className="w-4 h-4 mr-2" />
              Add Webhook
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {webhooks.map((webhook) => (
            <div key={webhook.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between mb-3">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{webhook.name}</span>
                    <Badge 
                      variant={webhook.status === "active" ? "default" : "secondary"}
                      className="text-xs"
                    >
                      {webhook.status}
                    </Badge>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {webhook.url}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    Last delivery: {webhook.lastDelivery}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="ghost" size="sm">
                    <RefreshCw className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm" className="text-destructive">
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
              <div className="flex flex-wrap gap-1">
                {webhook.events.map((event) => (
                  <Badge key={event} variant="outline" className="text-xs">
                    {event}
                  </Badge>
                ))}
              </div>
            </div>
          ))}

          {/* Add New Webhook Form */}
          <div className="p-4 rounded-lg border-2 border-dashed border-border">
            <h4 className="font-medium mb-3">Add New Webhook</h4>
            <div className="space-y-3">
              <div className="grid gap-3 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="webhookName">Webhook Name</Label>
                  <Input id="webhookName" placeholder="My Webhook" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="webhookUrl">Endpoint URL</Label>
                  <Input id="webhookUrl" placeholder="https://api.example.com/webhook" />
                </div>
              </div>
              <div className="space-y-2">
                <Label>Events</Label>
                <div className="grid gap-2 md:grid-cols-3">
                  {[
                    "repository.sync",
                    "repository.error",
                    "ai.request",
                    "ai.response",
                    "ai.error",
                    "user.login"
                  ].map((event) => (
                    <div key={event} className="flex items-center space-x-2">
                      <Switch id={`event-${event}`} />
                      <Label htmlFor={`event-${event}`} className="text-sm">
                        {event}
                      </Label>
                    </div>
                  ))}
                </div>
              </div>
            </div>
            <Button className="mt-3">
              <Webhook className="w-4 h-4 mr-2" />
              Create Webhook
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Feature Flags */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="w-5 h-5" />
            Feature Flags
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {featureFlags.map((flag) => (
            <div key={flag.name} className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/20">
              <div className="space-y-0.5">
                <div className="flex items-center gap-2">
                  <code className="text-sm font-mono bg-muted px-2 py-1 rounded">
                    {flag.name}
                  </code>
                  <Badge 
                    variant={flag.enabled ? "default" : "secondary"}
                    className="text-xs"
                  >
                    {flag.enabled ? "Enabled" : "Disabled"}
                  </Badge>
                </div>
                <p className="text-sm text-muted-foreground">
                  {flag.description}
                </p>
              </div>
              <Switch defaultChecked={flag.enabled} />
            </div>
          ))}
        </CardContent>
      </Card>

      {/* Custom Integrations */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="w-5 h-5" />
            Custom Integrations
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="customEndpoint">Custom API Endpoint</Label>
            <Input 
              id="customEndpoint" 
              placeholder="https://your-custom-api.com/v1"
            />
            <p className="text-sm text-muted-foreground">
              Override the default ConHub API endpoint for custom deployments
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="customHeaders">Custom Headers (JSON)</Label>
            <Textarea 
              id="customHeaders" 
              placeholder='{"Authorization": "Bearer your-token", "X-Custom-Header": "value"}'
              className="resize-none font-mono text-sm"
              rows={4}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Enable custom SSL certificates</Label>
              <p className="text-sm text-muted-foreground">
                Use custom SSL certificates for API connections
              </p>
            </div>
            <Switch />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Debug mode</Label>
              <p className="text-sm text-muted-foreground">
                Enable detailed logging for API requests and responses
              </p>
            </div>
            <Switch />
          </div>
        </CardContent>
      </Card>

      {/* Database & Storage */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="w-5 h-5" />
            Database & Storage
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="dbBackup">Backup Frequency</Label>
              <Select defaultValue="daily">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="hourly">Hourly</SelectItem>
                  <SelectItem value="daily">Daily</SelectItem>
                  <SelectItem value="weekly">Weekly</SelectItem>
                  <SelectItem value="monthly">Monthly</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="retentionPeriod">Data Retention Period</Label>
              <Select defaultValue="90">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="30">30 days</SelectItem>
                  <SelectItem value="90">90 days</SelectItem>
                  <SelectItem value="180">180 days</SelectItem>
                  <SelectItem value="365">1 year</SelectItem>
                  <SelectItem value="forever">Forever</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Automatic cleanup</Label>
              <p className="text-sm text-muted-foreground">
                Automatically remove old logs and temporary files
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Compress stored data</Label>
              <p className="text-sm text-muted-foreground">
                Use compression to reduce storage usage
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Button variant="outline" className="w-full">
            <Database className="w-4 h-4 mr-2" />
            Export All Data
          </Button>
        </CardContent>
      </Card>

      {/* Advanced Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="w-5 h-5" />
            Advanced Configuration
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="customConfig">Custom Configuration (JSON)</Label>
            <Textarea 
              id="customConfig" 
              placeholder='{"timeout": 30000, "retries": 3, "concurrency": 10}'
              className="resize-none font-mono text-sm"
              rows={6}
            />
            <p className="text-sm text-muted-foreground">
              Advanced configuration options for power users. Be careful with these settings.
            </p>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Experimental features</Label>
              <p className="text-sm text-muted-foreground">
                Enable experimental and unstable features
              </p>
            </div>
            <Switch />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Developer mode</Label>
              <p className="text-sm text-muted-foreground">
                Show additional debugging information and tools
              </p>
            </div>
            <Switch />
          </div>
        </CardContent>
      </Card>

      {/* Save Changes */}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Reset to Defaults</Button>
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
