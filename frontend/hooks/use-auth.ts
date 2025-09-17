'use client'

import { useAuth as useAuthContext } from '@/contexts/auth-context'
import { isLoginEnabled } from '@/lib/feature-toggles'

export const useAuth = () => {
  const loginEnabled = isLoginEnabled()
  const authContext = useAuthContext()
  
  // Mock user for when login is disabled
  const mockUser = {
    id: 'dev-user',
    name: 'Development User',
    email: 'dev@conhub.local',
    avatar_url: undefined,
    organization: 'ConHub Dev',
    role: 'admin' as const,
    subscription_tier: 'enterprise' as const,
    is_verified: true,
    created_at: new Date().toISOString(),
    last_login_at: new Date().toISOString(),
  }

  const login = (email?: string, password?: string) => {
    if (loginEnabled && email && password) {
      return authContext.login(email, password)
    } else {
      // For mock mode, redirect to dashboard
      window.location.href = '/dashboard'
      return Promise.resolve()
    }
  }

  const loginWithRedirect = () => {
    if (loginEnabled) {
      window.location.href = '/auth/login'
    } else {
      window.location.href = '/dashboard'
    }
  }

  const logoutUser = () => {
    if (loginEnabled) {
      authContext.logout()
    } else {
      // For mock mode, redirect to landing page
      window.location.href = '/'
    }
  }

  const getAccessTokenSilently = async () => {
    if (loginEnabled) {
      return authContext.token || 'mock-token'
    }
    return 'mock-token'
  }

  return {
    user: loginEnabled ? authContext.user : mockUser,
    isAuthenticated: loginEnabled ? authContext.isAuthenticated : true,
    isLoading: loginEnabled ? authContext.isLoading : false,
    login,
    loginWithRedirect,
    logout: logoutUser,
    getAccessTokenSilently,
    // Additional methods from new auth context
    register: loginEnabled ? authContext.register : undefined,
    updateProfile: loginEnabled ? authContext.updateProfile : undefined,
    changePassword: loginEnabled ? authContext.changePassword : undefined,
  }
}
