import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { 
  Shield, 
  Key, 
  Smartphone,
  Eye,
  Download,
  Trash2,
  Clock,
  MapPin,
  Monitor,
  AlertTriangle,
  Copy,
  Plus,
  X
} from "lucide-react";

export function SecuritySettings() {
  const activeSessions = [
    {
      id: 1,
      device: "MacBook Pro",
      location: "San Francisco, CA",
      ip: "192.168.1.100",
      lastActive: "2 minutes ago",
      current: true
    },
    {
      id: 2,
      device: "iPhone 15 Pro",
      location: "San Francisco, CA",
      ip: "192.168.1.101",
      lastActive: "1 hour ago",
      current: false
    },
    {
      id: 3,
      device: "Windows Desktop",
      location: "New York, NY",
      ip: "203.0.113.42",
      lastActive: "2 days ago",
      current: false
    }
  ];

  const apiTokens = [
    {
      id: 1,
      name: "Development API",
      token: "ch_dev_***********1234",
      created: "2024-08-15",
      lastUsed: "2 minutes ago",
      permissions: ["read", "write"]
    },
    {
      id: 2,
      name: "CI/CD Pipeline",
      token: "ch_cicd_**********5678",
      created: "2024-07-20",
      lastUsed: "1 hour ago",
      permissions: ["read"]
    },
    {
      id: 3,
      name: "Mobile App",
      token: "ch_mobile_********9012",
      created: "2024-06-10",
      lastUsed: "Never",
      permissions: ["read", "write"]
    }
  ];

  return (
    <div className="space-y-6">
      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="w-5 h-5" />
            Two-Factor Authentication
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 rounded-lg bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800">
            <div className="flex items-center gap-3">
              <Shield className="w-5 h-5 text-green-600" />
              <div>
                <p className="font-medium text-green-800 dark:text-green-200">2FA is enabled</p>
                <p className="text-sm text-green-600 dark:text-green-400">Your account is protected with two-factor authentication</p>
              </div>
            </div>
            <Badge variant="default" className="bg-green-600">Enabled</Badge>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label>Authenticator app</Label>
                <p className="text-sm text-muted-foreground">
                  Use an authenticator app like Google Authenticator or Authy
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Badge variant="default">Active</Badge>
                <Button variant="outline" size="sm">Reconfigure</Button>
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label>Backup codes</Label>
                <p className="text-sm text-muted-foreground">
                  Download recovery codes in case you lose your device
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Button variant="outline" size="sm">
                  <Download className="w-4 h-4 mr-2" />
                  Download
                </Button>
                <Button variant="outline" size="sm">Regenerate</Button>
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label>SMS backup</Label>
                <p className="text-sm text-muted-foreground">
                  Use SMS as a backup 2FA method (less secure)
                </p>
              </div>
              <Switch />
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="w-5 h-5" />
            Password & Authentication
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Change password</Label>
              <p className="text-sm text-muted-foreground">
                Last changed 3 months ago
              </p>
            </div>
            <Button variant="outline">Change Password</Button>
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Login notifications</Label>
              <p className="text-sm text-muted-foreground">
                Get notified of new login attempts
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Require 2FA for sensitive actions</Label>
              <p className="text-sm text-muted-foreground">
                Require 2FA when deleting repositories or changing security settings
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Session timeout</Label>
              <p className="text-sm text-muted-foreground">
                Automatically sign out after period of inactivity
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Input className="w-20 h-8" defaultValue="24" />
              <span className="text-sm text-muted-foreground">hours</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Monitor className="w-5 h-5" />
            Active Sessions
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {activeSessions.map((session) => (
            <div key={session.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Monitor className="w-5 h-5 text-muted-foreground" />
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{session.device}</span>
                      {session.current && <Badge variant="default">Current</Badge>}
                    </div>
                    <div className="text-sm text-muted-foreground flex items-center gap-4">
                      <span className="flex items-center gap-1">
                        <MapPin className="w-3 h-3" />
                        {session.location}
                      </span>
                      <span>{session.ip}</span>
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        {session.lastActive}
                      </span>
                    </div>
                  </div>
                </div>
                {!session.current && (
                  <Button variant="outline" size="sm" className="text-destructive">
                    <X className="w-4 h-4 mr-2" />
                    Revoke
                  </Button>
                )}
              </div>
            </div>
          ))}
          
          <Button variant="outline" className="w-full text-destructive">
            <AlertTriangle className="w-4 h-4 mr-2" />
            Sign out all other sessions
          </Button>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Key className="w-5 h-5" />
              API Tokens
            </CardTitle>
            <Button size="sm">
              <Plus className="w-4 h-4 mr-2" />
              Generate Token
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {apiTokens.map((token) => (
            <div key={token.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="font-medium">{token.name}</span>
                    <div className="flex gap-1">
                      {token.permissions.map((perm) => (
                        <Badge key={perm} variant="secondary" className="text-xs">
                          {perm}
                        </Badge>
                      ))}
                    </div>
                  </div>
                  <div className="text-sm text-muted-foreground space-y-1">
                    <div className="flex items-center gap-1">
                      <Key className="w-3 h-3" />
                      <code className="bg-muted px-2 py-1 rounded text-xs">{token.token}</code>
                      <Button variant="ghost" size="sm" className="h-6 w-6 p-0">
                        <Copy className="w-3 h-3" />
                      </Button>
                    </div>
                    <div className="flex items-center gap-4">
                      <span>Created: {token.created}</span>
                      <span>Last used: {token.lastUsed}</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm">
                    <Eye className="w-4 h-4 mr-2" />
                    View
                  </Button>
                  <Button variant="outline" size="sm" className="text-destructive">
                    <Trash2 className="w-4 h-4 mr-2" />
                    Delete
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertTriangle className="w-5 h-5" />
            Security Audit Log
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 rounded-lg bg-muted/20">
              <div className="flex items-center gap-3">
                <Shield className="w-4 h-4 text-green-600" />
                <div>
                  <p className="font-medium">Password changed</p>
                  <p className="text-sm text-muted-foreground">2 hours ago from 192.168.1.100</p>
                </div>
              </div>
            </div>
            <div className="flex items-center justify-between p-3 rounded-lg bg-muted/20">
              <div className="flex items-center gap-3">
                <Key className="w-4 h-4 text-blue-600" />
                <div>
                  <p className="font-medium">API token created</p>
                  <p className="text-sm text-muted-foreground">1 day ago</p>
                </div>
              </div>
            </div>
            <div className="flex items-center justify-between p-3 rounded-lg bg-muted/20">
              <div className="flex items-center gap-3">
                <Smartphone className="w-4 h-4 text-orange-600" />
                <div>
                  <p className="font-medium">New device login</p>
                  <p className="text-sm text-muted-foreground">3 days ago from iPhone</p>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}