'use client'

import { useEffect, useState } from 'react'
import { useSearchParams, useRouter } from 'next/navigation'
import { apiClient } from '@/lib/api'
import { useAuth } from '@/contexts/auth-context'

export default function AuthCallbackPage() {
  const params = useSearchParams()
  const router = useRouter()
  const { token } = useAuth()
  const [error, setError] = useState('')

  useEffect(() => {
    const provider = params.get('provider') || params.get('state') || ''
    const code = params.get('code') || ''
    if (!provider || !code) {
      setError('Missing provider or code')
      return
    }
    const run = async () => {
      try {
        const headers = token ? { Authorization: `Bearer ${token}` } : {}
        await apiClient.post('/api/auth/oauth/exchange', { provider, code }, headers)
        if (window.opener) {
          window.opener.postMessage({ type: 'oauth-connected', provider }, '*')
          window.close()
        } else {
          router.push('/dashboard/social')
        }
      } catch (e: any) {
        setError(e?.message || 'Failed to complete OAuth')
      }
    }
    run()
  }, [params, router, token])

  return (
    <div className="min-h-screen flex items-center justify-center">
      {error ? <div>{error}</div> : <div>Connecting…</div>}
    </div>
  )
}