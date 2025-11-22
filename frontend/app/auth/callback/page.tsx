'use client'

import { useEffect, useState } from 'react'
import { useSearchParams, useRouter } from 'next/navigation'
import { useAuth0 } from '@/contexts/auth0-context'

export default function AuthCallbackPage() {
  const params = useSearchParams()
  const router = useRouter()
  const { handleAuth0Callback } = useAuth0()
  const [error, setError] = useState('')
  const [processing, setProcessing] = useState(true)

  useEffect(() => {
    const code = params.get('code')
    const state = params.get('state')
    const errorParam = params.get('error')
    const errorDescription = params.get('error_description')

    // Check for Auth0 errors
    if (errorParam) {
      setError(errorDescription || errorParam)
      setProcessing(false)
      return
    }

    // Validate state for CSRF protection
    const savedState = localStorage.getItem('auth0_state')
    if (state !== savedState) {
      setError('Invalid state parameter - possible CSRF attack')
      setProcessing(false)
      return
    }

    if (!code) {
      setError('Missing authorization code')
      setProcessing(false)
      return
    }

    // Exchange authorization code for tokens
    const exchangeCodeForTokens = async () => {
      try {
        const auth0Domain = process.env.NEXT_PUBLIC_AUTH0_DOMAIN
        const auth0ClientId = process.env.NEXT_PUBLIC_AUTH0_CLIENT_ID
        const auth0Audience = process.env.NEXT_PUBLIC_AUTH0_AUDIENCE
        const auth0RedirectUri = process.env.NEXT_PUBLIC_AUTH0_REDIRECT_URI || 'http://localhost:3000/auth/callback'

        if (!auth0Domain || !auth0ClientId || !auth0Audience) {
          throw new Error('Auth0 configuration missing')
        }

        // Get code verifier from storage (PKCE)
        const codeVerifier = localStorage.getItem('auth0_code_verifier')
        if (!codeVerifier) {
          throw new Error('Missing code verifier')
        }

        // Exchange code for Auth0 tokens
        const tokenResponse = await fetch(`https://${auth0Domain}/oauth/token`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            grant_type: 'authorization_code',
            client_id: auth0ClientId,
            code: code,
            redirect_uri: auth0RedirectUri,
            code_verifier: codeVerifier,
            audience: auth0Audience
          })
        })

        if (!tokenResponse.ok) {
          const errorData = await tokenResponse.json()
          throw new Error(errorData.error_description || 'Token exchange failed')
        }

        const tokens = await tokenResponse.json()
        
        // Clean up temporary storage
        localStorage.removeItem('auth0_state')
        localStorage.removeItem('auth0_code_verifier')

        // Exchange Auth0 token for ConHub token
        await handleAuth0Callback(tokens.access_token)
      } catch (e: any) {
        console.error('Auth callback error:', e)
        setError(e?.message || 'Authentication failed')
        setProcessing(false)
      }
    }

    exchangeCodeForTokens()
  }, [params, handleAuth0Callback])

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
      <div className="text-center p-8 bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md">
        {processing && !error && (
          <>
            <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-600 mx-auto mb-4"></div>
            <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-2">
              Completing Sign In...
            </h2>
            <p className="text-gray-600 dark:text-gray-400">
              Please wait while we set up your account
            </p>
          </>
        )}
        
        {error && (
          <>
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
          </>
        )}
      </div>
    </div>
  )
}