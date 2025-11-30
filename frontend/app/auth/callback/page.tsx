'use client'

import { useEffect, useState } from 'react'
import { useSearchParams, useRouter } from 'next/navigation'
import { useAuth0 } from '@auth0/auth0-react'
import { apiClient } from '@/lib/api'
import { useAuth } from '@/hooks/use-auth'

export default function AuthCallbackPage() {
  const params = useSearchParams()
  const router = useRouter()
  const { isAuthenticated, isLoading, error: auth0Error, getAccessTokenSilently } = useAuth0()
  const { token } = useAuth()
  const [error, setError] = useState('')
  const [processing, setProcessing] = useState(true)
  const [exchanged, setExchanged] = useState(false)

  // Handle provider OAuth callback (GitHub, Google, etc. for connections, not login)
  useEffect(() => {
    const provider = params.get('provider')
    const code = params.get('code')
    const errorParam = params.get('error')
    const errorDescription = params.get('error_description')

    // Check for errors in URL
    if (errorParam) {
      setError(errorDescription || errorParam)
      setProcessing(false)
      return
    }

    // Provider OAuth exchange (for connecting external accounts, not Auth0 login)
    if (provider && code) {
      const exchangeProvider = async () => {
        try {
          const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {}
          await apiClient.post('/api/auth/oauth/exchange', { provider, code }, headers)
          try { window.opener?.postMessage({ type: 'oauth-connected', provider }, '*') } catch {}
          setProcessing(false)
          router.replace('/dashboard/connections')
        } catch (e: any) {
          console.error('OAuth exchange error:', e)
          setError(e?.message || 'OAuth exchange failed')
          setProcessing(false)
        }
      }
      exchangeProvider()
    }
  }, [params, token, router])

  // Handle Auth0 callback - the SDK handles the code exchange automatically
  // We just need to wait for authentication and then exchange for ConHub token
  useEffect(() => {
    const provider = params.get('provider')
    
    // Skip if this is a provider OAuth flow
    if (provider) return

    // Skip if already exchanged or still loading
    if (exchanged || isLoading) return

    // Handle Auth0 errors
    if (auth0Error) {
      setError(auth0Error.message)
      setProcessing(false)
      return
    }

    // Once Auth0 SDK has authenticated, exchange for ConHub token
    if (isAuthenticated) {
      const exchangeForConHubToken = async () => {
        try {
          // Get Auth0 access token
          const auth0Token = await getAccessTokenSilently()
          
          // Exchange Auth0 token for ConHub token
          const authServiceUrl = process.env.NEXT_PUBLIC_AUTH_SERVICE_URL || 'http://localhost:3010'
          const exchangeResponse = await fetch(`${authServiceUrl}/api/auth/auth0/exchange`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${auth0Token}`,
              'Content-Type': 'application/json'
            }
          })

          if (!exchangeResponse.ok) {
            const errorData = await exchangeResponse.json()
            throw new Error(errorData.message || 'ConHub token exchange failed')
          }

          const authResponse = await exchangeResponse.json()
          
          // Save session to localStorage
          const sessionData = {
            token: authResponse.token,
            refresh_token: authResponse.refresh_token,
            expires_at: authResponse.expires_at,
            last_activity: new Date().toISOString()
          }
          localStorage.setItem('auth_session', JSON.stringify(sessionData))
          localStorage.setItem('auth_token', authResponse.token)
          localStorage.setItem('refresh_token', authResponse.refresh_token)

          setExchanged(true)
          // Redirect to dashboard
          router.push('/dashboard')
        } catch (e: any) {
          console.error('ConHub token exchange error:', e)
          setError(e?.message || 'Authentication failed')
          setProcessing(false)
        }
      }

      exchangeForConHubToken()
    }
  }, [isAuthenticated, isLoading, auth0Error, getAccessTokenSilently, router, exchanged, params])

  // Show loading while Auth0 SDK processes the callback
  if (isLoading || (processing && !error)) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
        <div className="text-center p-8 bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">Completing authentication...</p>
        </div>
      </div>
    )
  }

  // Show error if any
  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
        <div className="text-center p-8 bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md">
          <div className="text-red-500 text-5xl mb-4">⚠️</div>
          <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-2">
            Authentication Error
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mb-4">{error}</p>
          <button
            onClick={() => router.push('/')}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Return to Home
          </button>
        </div>
      </div>
    )
  }

  return null
}
