'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Bell, Mail, Smartphone, AlertCircle, CheckCircle } from 'lucide-react'
import { useToast } from '@/hooks/use-toast'

interface NotificationPreferences {
  email_notifications: boolean
  push_notifications: boolean
  security_alerts: boolean
  billing_notifications: boolean
  product_updates: boolean
  marketing_emails: boolean
  frequency: 'immediate' | 'daily' | 'weekly' | 'never'
}

export function NotificationSettings() {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)
  const [preferences, setPreferences] = useState<NotificationPreferences>({
    email_notifications: true,
    push_notifications: false,
    security_alerts: true,
    billing_notifications: true,
    product_updates: true,
    marketing_emails: false,
    frequency: 'immediate',
  })

  const handleToggle = (key: keyof NotificationPreferences, value: boolean | string) => {
    setPreferences(prev => ({ ...prev, [key]: value }))
  }

  const handleSave = async () => {
    setLoading(true)
    try {
      // TODO: Implement API call to save notification preferences
      await new Promise(resolve => setTimeout(resolve, 1000)) // Simulate API call
      
      toast({
        title: 'Settings Saved',
        description: 'Your notification preferences have been updated.',
      })
    } catch (error) {
      toast({
        title: 'Save Failed',
        description: 'Failed to save notification settings. Please try again.',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Notification Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bell className="h-5 w-5" />
            Notification Preferences
          </CardTitle>
          <CardDescription>
            Choose how and when you want to receive notifications
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            {/* Email Notifications */}
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <Mail className="h-5 w-5" />
                <h3 className="font-medium">Email Notifications</h3>
              </div>
              
              <div className="space-y-3 pl-7">
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="email-notifications">General Email Notifications</Label>
                    <p className="text-sm text-muted-foreground">
                      Receive email notifications for important updates
                    </p>
                  </div>
                  <Switch
                    id="email-notifications"
                    checked={preferences.email_notifications}
                    onCheckedChange={(checked) => handleToggle('email_notifications', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="security-alerts">Security Alerts</Label>
                    <p className="text-sm text-muted-foreground">
                      Important security notifications (always enabled)
                    </p>
                  </div>
                  <Switch
                    id="security-alerts"
                    checked={preferences.security_alerts}
                    onCheckedChange={(checked) => handleToggle('security_alerts', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="billing-notifications">Billing Notifications</Label>
                    <p className="text-sm text-muted-foreground">
                      Invoices, payment confirmations, and billing updates
                    </p>
                  </div>
                  <Switch
                    id="billing-notifications"
                    checked={preferences.billing_notifications}
                    onCheckedChange={(checked) => handleToggle('billing_notifications', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="product-updates">Product Updates</Label>
                    <p className="text-sm text-muted-foreground">
                      New features, improvements, and announcements
                    </p>
                  </div>
                  <Switch
                    id="product-updates"
                    checked={preferences.product_updates}
                    onCheckedChange={(checked) => handleToggle('product_updates', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="marketing-emails">Marketing Emails</Label>
                    <p className="text-sm text-muted-foreground">
                      Tips, tutorials, and promotional content
                    </p>
                  </div>
                  <Switch
                    id="marketing-emails"
                    checked={preferences.marketing_emails}
                    onCheckedChange={(checked) => handleToggle('marketing_emails', checked)}
                  />
                </div>
              </div>
            </div>

            {/* Push Notifications */}
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <Smartphone className="h-5 w-5" />
                <h3 className="font-medium">Push Notifications</h3>
              </div>
              
              <div className="space-y-3 pl-7">
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="push-notifications">Browser Push Notifications</Label>
                    <p className="text-sm text-muted-foreground">
                      Receive push notifications in your browser
                    </p>
                  </div>
                  <Switch
                    id="push-notifications"
                    checked={preferences.push_notifications}
                    onCheckedChange={(checked) => handleToggle('push_notifications', checked)}
                  />
                </div>

                {!preferences.push_notifications && (
                  <div className="flex items-center gap-2 p-3 bg-blue-50 border border-blue-200 rounded-lg">
                    <AlertCircle className="h-4 w-4 text-blue-500" />
                    <span className="text-sm text-blue-700">
                      Enable push notifications to get real-time updates
                    </span>
                  </div>
                )}
              </div>
            </div>

            {/* Notification Frequency */}
            <div className="space-y-4">
              <h3 className="font-medium">Notification Frequency</h3>
              <div className="pl-7">
                <div className="space-y-2">
                  <Label htmlFor="frequency">Email Digest Frequency</Label>
                  <Select
                    value={preferences.frequency}
                    onValueChange={(value) => handleToggle('frequency', value)}
                  >
                    <SelectTrigger className="w-48">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="immediate">Immediate</SelectItem>
                      <SelectItem value="daily">Daily Digest</SelectItem>
                      <SelectItem value="weekly">Weekly Digest</SelectItem>
                      <SelectItem value="never">Never</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-sm text-muted-foreground">
                    Choose how often you want to receive email notifications
                  </p>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Notification Categories */}
      <Card>
        <CardHeader>
          <CardTitle>Notification Categories</CardTitle>
          <CardDescription>
            What types of notifications you'll receive
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="p-4 border rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <h4 className="font-medium">Account & Security</h4>
                </div>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• Login attempts</li>
                  <li>• Password changes</li>
                  <li>• Account verification</li>
                  <li>• Security alerts</li>
                </ul>
              </div>

              <div className="p-4 border rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <h4 className="font-medium">Billing & Subscriptions</h4>
                </div>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• Payment confirmations</li>
                  <li>• Invoice notifications</li>
                  <li>• Subscription changes</li>
                  <li>• Usage limit alerts</li>
                </ul>
              </div>

              <div className="p-4 border rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <h4 className="font-medium">Product Updates</h4>
                </div>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• New features</li>
                  <li>• System maintenance</li>
                  <li>• Performance improvements</li>
                  <li>• Bug fixes</li>
                </ul>
              </div>

              <div className="p-4 border rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <h4 className="font-medium">Repository Activity</h4>
                </div>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• Sync completions</li>
                  <li>• Indexing updates</li>
                  <li>• Connection issues</li>
                  <li>• Webhook events</li>
                </ul>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Save Button */}
      <div className="flex justify-end">
        <Button onClick={handleSave} disabled={loading}>
          {loading ? 'Saving...' : 'Save Preferences'}
        </Button>
      </div>
    </div>
  )
}