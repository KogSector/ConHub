import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { 
  Bell, 
  Mail, 
  MessageSquare,
  Smartphone,
  Settings,
  Volume2,
  VolumeX,
  AlertCircle,
  CheckCircle,
  XCircle,
  Activity
} from "lucide-react";

export function NotificationSettings() {
  return (
    <div className="space-y-6">
      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Mail className="w-5 h-5" />
            Email Notifications
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Repository sync status</Label>
              <p className="text-sm text-muted-foreground">
                Get notified when repositories sync successfully or fail
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>AI agent errors</Label>
              <p className="text-sm text-muted-foreground">
                Receive alerts when AI agents encounter errors
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Weekly summary</Label>
              <p className="text-sm text-muted-foreground">
                Get a weekly summary of your ConHub activity
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Security alerts</Label>
              <p className="text-sm text-muted-foreground">
                Important security notifications and login alerts
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Product updates</Label>
              <p className="text-sm text-muted-foreground">
                New features, updates, and announcements
              </p>
            </div>
            <Switch />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Marketing communications</Label>
              <p className="text-sm text-muted-foreground">
                Tips, tutorials, and promotional content
              </p>
            </div>
            <Switch />
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bell className="w-5 h-5" />
            In-App Notifications
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Real-time sync updates</Label>
              <p className="text-sm text-muted-foreground">
                Show notifications when repositories are syncing
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>AI agent responses</Label>
              <p className="text-sm text-muted-foreground">
                Notify when AI agents complete requests
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>System maintenance</Label>
              <p className="text-sm text-muted-foreground">
                Alerts about scheduled maintenance and downtime
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Usage warnings</Label>
              <p className="text-sm text-muted-foreground">
                Warnings when approaching plan limits
              </p>
            </div>
            <Switch defaultChecked />
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Smartphone className="w-5 h-5" />
            Push Notifications
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Enable push notifications</Label>
              <p className="text-sm text-muted-foreground">
                Receive notifications on your devices even when ConHub is closed
              </p>
            </div>
            <Switch />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Critical alerts only</Label>
              <p className="text-sm text-muted-foreground">
                Only send push notifications for critical issues
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Quiet hours</Label>
              <p className="text-sm text-muted-foreground">
                Disable push notifications during specified hours
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Select defaultValue="22">
                <SelectTrigger className="w-20">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Array.from({ length: 24 }, (_, i) => (
                    <SelectItem key={i} value={i.toString()}>
                      {i.toString().padStart(2, '0')}:00
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <span className="text-sm text-muted-foreground">to</span>
              <Select defaultValue="8">
                <SelectTrigger className="w-20">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Array.from({ length: 24 }, (_, i) => (
                    <SelectItem key={i} value={i.toString()}>
                      {i.toString().padStart(2, '0')}:00
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <MessageSquare className="w-5 h-5" />
            Notification Channels
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="slack-webhook">Slack Webhook URL</Label>
            <Input 
              id="slack-webhook" 
              placeholder="https://hooks.slack.com/services/..." 
            />
            <p className="text-sm text-muted-foreground">
              Send notifications to your Slack workspace
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="discord-webhook">Discord Webhook URL</Label>
            <Input 
              id="discord-webhook" 
              placeholder="https://discord.com/api/webhooks/..." 
            />
            <p className="text-sm text-muted-foreground">
              Send notifications to your Discord server
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="teams-webhook">Microsoft Teams Webhook URL</Label>
            <Input 
              id="teams-webhook" 
              placeholder="https://outlook.office.com/webhook/..." 
            />
            <p className="text-sm text-muted-foreground">
              Send notifications to your Teams channel
            </p>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="w-5 h-5" />
            Notification Preferences
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Notification frequency</Label>
              <p className="text-sm text-muted-foreground">
                How often should we send notification summaries
              </p>
            </div>
            <Select defaultValue="immediate">
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="immediate">Immediate</SelectItem>
                <SelectItem value="hourly">Hourly digest</SelectItem>
                <SelectItem value="daily">Daily digest</SelectItem>
                <SelectItem value="weekly">Weekly digest</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Sound notifications</Label>
              <p className="text-sm text-muted-foreground">
                Play sound effects for in-app notifications
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Switch defaultChecked />
              <Volume2 className="w-4 h-4 text-muted-foreground" />
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Desktop badges</Label>
              <p className="text-sm text-muted-foreground">
                Show notification count on app icon
              </p>
            </div>
            <Switch defaultChecked />
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="w-5 h-5" />
            Notification Types
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="p-3 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="w-4 h-4 text-green-500" />
                <span className="font-medium text-sm">Success</span>
              </div>
              <p className="text-xs text-muted-foreground mb-2">
                Repository syncs, successful deployments, agent connections
              </p>
              <Switch defaultChecked />
            </div>

            <div className="p-3 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center gap-2 mb-2">
                <AlertCircle className="w-4 h-4 text-yellow-500" />
                <span className="font-medium text-sm">Warning</span>
              </div>
              <p className="text-xs text-muted-foreground mb-2">
                Approaching limits, deprecated features, slow responses
              </p>
              <Switch defaultChecked />
            </div>

            <div className="p-3 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center gap-2 mb-2">
                <XCircle className="w-4 h-4 text-red-500" />
                <span className="font-medium text-sm">Error</span>
              </div>
              <p className="text-xs text-muted-foreground mb-2">
                Failed syncs, agent errors, authentication issues
              </p>
              <Switch defaultChecked />
            </div>

            <div className="p-3 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center gap-2 mb-2">
                <Bell className="w-4 h-4 text-blue-500" />
                <span className="font-medium text-sm">Information</span>
              </div>
              <p className="text-xs text-muted-foreground mb-2">
                Feature updates, tips, maintenance schedules
              </p>
              <Switch />
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Test Notifications</Button>
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
