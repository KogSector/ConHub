'use client'

import { useAuth0 } from '@auth0/auth0-react'

export const useAuth = () => {
  const {
    user,
    isAuthenticated,
    isLoading,
    loginWithRedirect,
    logout,
    getAccessTokenSilently,
  } = useAuth0()

  const login = () => {
    loginWithRedirect({
      authorizationParams: {
        screen_hint: 'signup',
      },
    })
  }

  const logoutUser = () => {
    logout({
      logoutParams: {
        returnTo: window.location.origin,
      },
    })
  }

  return {
    user,
    isAuthenticated,
    isLoading,
    login,
    loginWithRedirect,
    logout: logoutUser,
    getAccessTokenSilently,
  }
}
