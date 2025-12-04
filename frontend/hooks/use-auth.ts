'use client'

import { useEffect, useState, useCallback } from 'react'
import { useAuth0 } from '@auth0/auth0-react'

export const useAuth = () => {
  const auth0 = useAuth0()
  
  // Auth0 access token state
  const [accessToken, setAccessToken] = useState<string | null>(null)
  const [tokenLoading, setTokenLoading] = useState(false)

  // Fetch and cache Auth0 access token when authenticated
  useEffect(() => {
    if (auth0.isAuthenticated && !auth0.isLoading && !accessToken && !tokenLoading) {
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
  }, [auth0.isAuthenticated, auth0.isLoading, accessToken, tokenLoading, auth0])

  const login = () => {
    return auth0.loginWithRedirect()
  }

  const loginWithRedirect = () => {
    auth0.loginWithRedirect()
  }

  const logoutUser = () => {
    auth0.logout({ logoutParams: { returnTo: window.location.origin } })
  }

  const getAccessTokenSilently = useCallback(async () => {
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
  }, [accessToken, auth0])

  // Clear token on logout
  useEffect(() => {
    if (!auth0.isAuthenticated && accessToken) {
      setAccessToken(null)
    }
  }, [auth0.isAuthenticated, accessToken])

  return {
    user: auth0.user as any,
    isAuthenticated: auth0.isAuthenticated,
    isLoading: auth0.isLoading || tokenLoading,
    login,
    loginWithRedirect,
    logout: logoutUser,
    getAccessTokenSilently,
    // Auth0 access token (use this for API calls)
    token: accessToken,
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
 
