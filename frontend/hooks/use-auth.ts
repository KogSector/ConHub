'use client'

import { useEffect, useState, useCallback } from 'react'
import { useAuth0 } from '@auth0/auth0-react'
import { isLoginEnabled } from '@/lib/feature-toggles'
import { fetchCurrentUserViaGraphQL } from '@/lib/api'

export const useAuth = () => {
  const loginEnabled = isLoginEnabled()
  const auth0 = useAuth0()
  
  // Auth0 access token state
  const [accessToken, setAccessToken] = useState<string | null>(null)
  const [tokenLoading, setTokenLoading] = useState(false)
  
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

  // Fetch and cache Auth0 access token when authenticated
  useEffect(() => {
    if (loginEnabled && auth0.isAuthenticated && !auth0.isLoading && !accessToken && !tokenLoading) {
      setTokenLoading(true)
      auth0.getAccessTokenSilently()
        .then((token) => {
          setAccessToken(token)
        })
        .catch((err) => {
          console.error('Failed to get Auth0 access token:', err)
          setAccessToken(null)
        })
        .finally(() => {
          setTokenLoading(false)
        })
    }
  }, [loginEnabled, auth0.isAuthenticated, auth0.isLoading, accessToken, tokenLoading, auth0])

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

  const getAccessTokenSilently = useCallback(async () => {
    if (loginEnabled) {
      // Return cached token if available
      if (accessToken) {
        return accessToken
      }
      // Otherwise fetch fresh
      try {
        if (auth0.getAccessTokenSilently) {
          const token = await auth0.getAccessTokenSilently()
          setAccessToken(token)
          return token
        }
      } catch (err) {
        console.error('getAccessTokenSilently failed:', err)
      }
      return null
    }
    return 'mock-token'
  }, [loginEnabled, accessToken, auth0])

  // Clear token on logout
  useEffect(() => {
    if (loginEnabled && !auth0.isAuthenticated && accessToken) {
      setAccessToken(null)
    }
  }, [loginEnabled, auth0.isAuthenticated, accessToken])

  const effectiveUser = loginEnabled
    ? (auth0.user as any) ?? mockUser
    : mockUser

  return {
    user: effectiveUser,
    isAuthenticated: loginEnabled ? auth0.isAuthenticated : true,
    isLoading: loginEnabled ? (auth0.isLoading || tokenLoading) : false,
    login,
    loginWithRedirect,
    logout: logoutUser,
    getAccessTokenSilently,
    // Auth0 access token (use this for API calls)
    token: loginEnabled ? accessToken : 'mock-token',
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
 
