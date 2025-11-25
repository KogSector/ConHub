'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { useAuth } from '@/hooks/use-auth'
import { useAuth0 } from '@auth0/auth0-react'
import { User, Building, Mail, Eye, EyeOff, Lock } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'

export function ProfileManagement() {
  const { user, updateProfile } = useAuth()
  const { loginWithRedirect } = useAuth0()
  const [isEditingProfile, setIsEditingProfile] = useState(false)
  const [isChangingPassword, setIsChangingPassword] = useState(false)
  const [profileData, setProfileData] = useState({
    name: '',
    organization: '',
    avatar_url: ''
  })
  const [showPasswords, setShowPasswords] = useState({
    current: false,
    new: false,
    confirm: false
  })
  const [message, setMessage] = useState('')
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  useEffect(() => {
    if (user) {
      setProfileData({
        name: user.name || '',
        organization: user.organization || '',
        avatar_url: user.avatar_url || ''
      })
    }
  }, [user])

  const handleProfileSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!updateProfile) return

    setIsLoading(true)
    setError('')
    setMessage('')

    try {
      await updateProfile(profileData)
      setMessage('Profile updated successfully!')
      setIsEditingProfile(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update profile')
    } finally {
      setIsLoading(false)
    }
  }

  const handlePasswordReset = () => {
    loginWithRedirect({
      authorizationParams: {
        screen_hint: 'forgot_password',
      }
    })
  }

  if (!user) {
    return (
      <div className="flex items-center justify-center p-8">
        <p className="text-gray-600">Please log in to view your profile.</p>
      </div>
    )
  }

  const userInitials = user.name
    .split(' ')
    .map((n: string) => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)

  return (
    <div className="max-w-2xl mx-auto p-6 space-y-6">
      {(message || error) && (
        <div className={`p-3 text-sm rounded-md ${
          message 
            ? 'text-green-600 bg-green-50 border border-green-200' 
            : 'text-red-600 bg-red-50 border border-red-200'
        }`}>
          {message || error}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            Profile Information
          </CardTitle>
          <CardDescription>
            Manage your account details and preferences
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center gap-4">
            <Avatar className="h-16 w-16">
              <AvatarImage src={user.avatar_url} alt={user.name} />
              <AvatarFallback className="text-lg">{userInitials}</AvatarFallback>
            </Avatar>
            <div>
              <h3 className="font-medium">{user.name}</h3>
              <p className="text-sm text-gray-600">{user.email}</p>
              <p className="text-xs text-gray-500 capitalize">
                {user.subscription_tier} • {user.role}
              </p>
            </div>
          </div>

          <Separator />

          {isEditingProfile ? (
            <form onSubmit={handleProfileSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="profile-name">Full Name</Label>
                <Input
                  id="profile-name"
                  value={profileData.name}
                  onChange={(e) => setProfileData(prev => ({ ...prev, name: e.target.value }))}
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="profile-organization">Organization</Label>
                <Input
                  id="profile-organization"
                  value={profileData.organization}
                  onChange={(e) => setProfileData(prev => ({ ...prev, organization: e.target.value }))}
                  placeholder="Optional"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="profile-avatar">Profile Picture URL</Label>
                <Input
                  id="profile-avatar"
                  type="url"
                  value={profileData.avatar_url}
                  onChange={(e) => setProfileData(prev => ({ ...prev, avatar_url: e.target.value }))}
                  placeholder="https://example.com/avatar.jpg"
                />
              </div>

              <div className="flex gap-2">
                <Button type="submit" disabled={isLoading}>
                  {isLoading ? 'Saving...' : 'Save Changes'}
                </Button>
                <Button 
                  type="button" 
                  variant="outline" 
                  onClick={() => {
                    setIsEditingProfile(false)
                    setProfileData({
                      name: user.name || '',
                      organization: user.organization || '',
                      avatar_url: user.avatar_url || ''
                    })
                  }}
                >
                  Cancel
                </Button>
              </div>
            </form>
          ) : (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-sm font-medium text-gray-600">Email</Label>
                  <p className="flex items-center gap-2 mt-1">
                    <Mail className="h-4 w-4 text-gray-400" />
                    {user.email}
                  </p>
                </div>
                <div>
                  <Label className="text-sm font-medium text-gray-600">Organization</Label>
                  <p className="flex items-center gap-2 mt-1">
                    <Building className="h-4 w-4 text-gray-400" />
                    {user.organization || 'Not specified'}
                  </p>
                </div>
              </div>
              <Button onClick={() => setIsEditingProfile(true)}>
                Edit Profile
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lock className="h-5 w-5" />
            Security
          </CardTitle>
          <CardDescription>
            Update your password to keep your account secure
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isChangingPassword ? (
            <div className="space-y-4">
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Click the button below to reset your password. You'll be redirected to a secure page to complete the process.
              </p>
              <div className="flex justify-end">
                <Button 
                  type="button" 
                  onClick={handlePasswordReset}
                  variant="outline"
                >
                  Reset Password
                </Button>
              </div>
            </div>
          ) : (
            <Button onClick={() => setIsChangingPassword(true)}>
              Change Password
            </Button>
          )}
        </CardContent>
      </Card>
    </div>
  )
}