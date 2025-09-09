'use client'

import { useAuth0 } from '@auth0/auth0-react'
import { isLoginEnabled } from '@/lib/feature-toggles'

export const useAuth = () => {
  const loginEnabled = isLoginEnabled()
  
  const auth0 = useAuth0()
  
  // Mock user for when login is disabled
  const mockUser = {
    name: 'Development User',
    email: 'dev@conhub.local',
    picture: undefined
  }

  const login = () => {
    if (loginEnabled) {
      auth0.loginWithRedirect({
        authorizationParams: {
          screen_hint: 'signup',
        },
      })
    }
  }

  const logoutUser = () => {
    if (loginEnabled) {
      auth0.logout({
        logoutParams: {
          returnTo: `${window.location.origin}/`,
        },
      })
    } else {
      // For mock mode, redirect to landing page
      window.location.href = '/'
    }
  }

  return {
    user: loginEnabled ? auth0.user : mockUser,
    isAuthenticated: loginEnabled ? auth0.isAuthenticated : true,
    isLoading: loginEnabled ? auth0.isLoading : false,
    login,
    loginWithRedirect: login,
    logout: logoutUser,
    getAccessTokenSilently: loginEnabled ? auth0.getAccessTokenSilently : async () => 'mock-token',
  }
}
