'use client'

import { useState } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { AlertTriangle, Trash2, Download, Shield } from 'lucide-react'
import { useToast } from '@/hooks/use-toast'

export function AccountDeletion() {
  const { user, logout } = useAuth()
  const { toast } = useToast()
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [confirmationText, setConfirmationText] = useState('')
  const [acknowledgeDataLoss, setAcknowledgeDataLoss] = useState(false)
  const [acknowledgeBilling, setAcknowledgeBilling] = useState(false)
  const [deleting, setDeleting] = useState(false)

  const expectedConfirmation = `DELETE ${user?.email}`
  const canDelete = confirmationText === expectedConfirmation && acknowledgeDataLoss && acknowledgeBilling

  const handleDeleteAccount = async () => {
    if (!canDelete) return

    setDeleting(true)
    try {
      // TODO: Implement account deletion API call
      await new Promise(resolve => setTimeout(resolve, 2000)) // Simulate API call
      
      toast({
        title: 'Account Deleted',
        description: 'Your account has been permanently deleted.',
      })
      
      // Log out the user
      logout()
    } catch (error) {
      toast({
        title: 'Deletion Failed',
        description: 'Failed to delete account. Please contact support.',
        variant: 'destructive',
      })
    } finally {
      setDeleting(false)
      setShowDeleteDialog(false)
    }
  }

  const handleExportData = async () => {
    try {
      // TODO: Implement data export API call
      toast({
        title: 'Export Started',
        description: 'Your data export has been initiated. You will receive an email when ready.',
      })
    } catch (error) {
      toast({
        title: 'Export Failed',
        description: 'Failed to start data export. Please try again.',
        variant: 'destructive',
      })
    }
  }

  return (
    <div className="space-y-6">
      {/* Warning Card */}
      <Card className="border-destructive">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-destructive">
            <AlertTriangle className="h-5 w-5" />
            Danger Zone
          </CardTitle>
          <CardDescription>
            Irreversible and destructive actions for your account
          </CardDescription>
        </CardHeader>
      </Card>

      {/* Data Export */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            Export Your Data
          </CardTitle>
          <CardDescription>
            Download a copy of your account data before deletion
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <h4 className="font-medium mb-2">What's included in your export:</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>• Profile information and settings</li>
                <li>• Repository connections and configurations</li>
                <li>• AI agent settings and usage history</li>
                <li>• Billing and subscription history</li>
                <li>• Document uploads and metadata</li>
              </ul>
            </div>
            
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-medium">Request Data Export</h3>
                <p className="text-sm text-muted-foreground">
                  We'll email you a download link when your export is ready
                </p>
              </div>
              <Button onClick={handleExportData} variant="outline">
                <Download className="h-4 w-4 mr-2" />
                Export Data
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Account Deactivation */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Deactivate Account
          </CardTitle>
          <CardDescription>
            Temporarily disable your account (reversible)
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="p-4 bg-orange-50 border border-orange-200 rounded-lg">
              <h4 className="font-medium mb-2">Account deactivation will:</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>• Hide your profile and data</li>
                <li>• Pause your subscription billing</li>
                <li>• Disable API access and integrations</li>
                <li>• Allow reactivation within 30 days</li>
              </ul>
            </div>
            
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-medium">Deactivate Account</h3>
                <p className="text-sm text-muted-foreground">
                  You can reactivate your account by logging in again
                </p>
              </div>
              <Button variant="outline">
                Deactivate Account
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Account Deletion */}
      <Card className="border-destructive">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-destructive">
            <Trash2 className="h-5 w-5" />
            Delete Account
          </CardTitle>
          <CardDescription>
            Permanently delete your account and all associated data
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
              <h4 className="font-medium mb-2 text-destructive">This action cannot be undone!</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>• All your data will be permanently deleted</li>
                <li>• Repository connections will be removed</li>
                <li>• AI agent configurations will be lost</li>
                <li>• Billing history will be archived (required by law)</li>
                <li>• Your username will become available to others</li>
              </ul>
            </div>
            
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-medium">Delete Account</h3>
                <p className="text-sm text-muted-foreground">
                  This will permanently delete your account and all data
                </p>
              </div>
              <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
                <DialogTrigger asChild>
                  <Button variant="destructive">
                    <Trash2 className="h-4 w-4 mr-2" />
                    Delete Account
                  </Button>
                </DialogTrigger>
                <DialogContent className="max-w-md">
                  <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-destructive">
                      <AlertTriangle className="h-5 w-5" />
                      Delete Account
                    </DialogTitle>
                    <DialogDescription>
                      This action cannot be undone. Please confirm you want to permanently delete your account.
                    </DialogDescription>
                  </DialogHeader>
                  
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="confirmation">
                        Type <code className="bg-muted px-1 rounded">{expectedConfirmation}</code> to confirm:
                      </Label>
                      <Input
                        id="confirmation"
                        value={confirmationText}
                        onChange={(e) => setConfirmationText(e.target.value)}
                        placeholder={expectedConfirmation}
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center space-x-2">
                        <Checkbox
                          id="acknowledge-data"
                          checked={acknowledgeDataLoss}
                          onCheckedChange={(checked) => setAcknowledgeDataLoss(checked as boolean)}
                        />
                        <Label htmlFor="acknowledge-data" className="text-sm">
                          I understand that all my data will be permanently deleted
                        </Label>
                      </div>

                      <div className="flex items-center space-x-2">
                        <Checkbox
                          id="acknowledge-billing"
                          checked={acknowledgeBilling}
                          onCheckedChange={(checked) => setAcknowledgeBilling(checked as boolean)}
                        />
                        <Label htmlFor="acknowledge-billing" className="text-sm">
                          I understand that active subscriptions will be cancelled
                        </Label>
                      </div>
                    </div>

                    <div className="flex justify-end space-x-2">
                      <Button
                        variant="outline"
                        onClick={() => setShowDeleteDialog(false)}
                        disabled={deleting}
                      >
                        Cancel
                      </Button>
                      <Button
                        variant="destructive"
                        onClick={handleDeleteAccount}
                        disabled={!canDelete || deleting}
                      >
                        {deleting ? 'Deleting...' : 'Delete Account'}
                      </Button>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}