'use client'

import { useEffect, useState } from 'react'
import { useAuth as useAuthContext } from '@/contexts/auth-context'
import { isLoginEnabled } from '@/lib/feature-toggles'
import { fetchCurrentUserViaGraphQL } from '@/lib/api'

export const useAuth = () => {
  const loginEnabled = isLoginEnabled()
  const authContext = useAuthContext()
  
  const [mockUser, setMockUser] = useState({
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
  })

  useEffect(() => {
    if (!loginEnabled) {
      // Attempt to hydrate mock user from backend GraphQL claims
      fetchCurrentUserViaGraphQL()
        .then((me) => {
          if (me) {
            const roles = Array.isArray(me.roles) ? me.roles.map(r => r.toLowerCase()) : []
            const mappedRole = roles.includes('admin') ? 'admin' : roles.includes('moderator') ? 'moderator' : 'user'
            setMockUser(prev => ({
              ...prev,
              id: me.user_id ?? prev.id,
              role: mappedRole as typeof prev.role,
            }))
          }
        })
        .catch(() => {
          // Keep default dev user if GraphQL isn't reachable
        })
    }
  }, [loginEnabled])

  const login = (email?: string, password?: string) => {
    if (loginEnabled && email && password) {
      return authContext.login(email, password)
    } else {
      
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
    
    register: loginEnabled ? authContext.register : undefined,
    updateProfile: loginEnabled ? authContext.updateProfile : undefined,
    changePassword: loginEnabled ? authContext.changePassword : undefined,
  }
}
