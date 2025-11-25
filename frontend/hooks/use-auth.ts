'use client'

import { useEffect, useState } from 'react'
import { useAuth0 } from '@auth0/auth0-react'
import { isLoginEnabled } from '@/lib/feature-toggles'
import { fetchCurrentUserViaGraphQL } from '@/lib/api'

export const useAuth = () => {
  const loginEnabled = isLoginEnabled()
  const auth0 = useAuth0()
  
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
    if (loginEnabled) {
      // Delegate to Auth0 Universal Login; email/password are handled by Auth0
      return auth0.loginWithRedirect()
    } else {
      window.location.href = '/dashboard'
      return Promise.resolve()
    }
  }

  const loginWithRedirect = () => {
    if (loginEnabled) {
      auth0.loginWithRedirect()
    } else {
      window.location.href = '/dashboard'
    }
  }

  const logoutUser = () => {
    if (loginEnabled) {
      auth0.logout({ logoutParams: { returnTo: window.location.origin } })
    } else {
      window.location.href = '/'
    }
  }

  const getAccessTokenSilently = async () => {
    if (loginEnabled) {
      try {
        if (auth0.getAccessTokenSilently) {
          return await auth0.getAccessTokenSilently()
        }
      } catch {
        // fall through to mock token
      }
      return 'mock-token'
    }
    return 'mock-token'
  }

  const effectiveUser = loginEnabled
    ? (auth0.user as any) ?? mockUser
    : mockUser

  return {
    user: effectiveUser,
    isAuthenticated: loginEnabled ? auth0.isAuthenticated : true,
    isLoading: loginEnabled ? auth0.isLoading : false,
    login,
    loginWithRedirect,
    logout: logoutUser,
    getAccessTokenSilently,
    // Placeholder token and connections fields to satisfy existing callers.
    // For real backend calls, use getAccessTokenSilently instead.
    token: null as string | null,
    connections: null as any,

    // Stubbed methods for now; callers may override these with real implementations
    // when backend Auth0-backed profile/password flows are wired up.
    register: async (_data: {
      email: string
      password: string
      name: string
      organization?: string
      avatar_url?: string
    }) => {
      throw new Error('register is not implemented for Auth0-based auth yet')
    },
    updateProfile: async (_data: {
      name?: string
      email?: string
      organization?: string
      avatar_url?: string
    }) => {
      throw new Error('updateProfile is not implemented for Auth0-based auth yet')
    },
    changePassword: async () => {
      throw new Error('changePassword is not implemented for Auth0-based auth yet')
    },
  }
}
 
