'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { useAuth } from '@/hooks/use-auth'
import { isLoginEnabled } from '@/lib/feature-toggles'

interface AuthGuardProps {
  children: React.ReactNode
  requireAuth?: boolean
  redirectTo?: string
}

export function AuthGuard({
  children,
  requireAuth = true,
  redirectTo = '/auth/login'
}: AuthGuardProps) {
  const { isAuthenticated, isLoading } = useAuth()
  const router = useRouter()

  // If login is disabled, bypass auth requirements entirely
  const authRequired = requireAuth && isLoginEnabled()
  console.log('[AuthGuard]', { authRequired, isAuthenticated, isLoading, redirectTo })

  useEffect(() => {
    if (authRequired && !isAuthenticated) {
      router.push(redirectTo)
    }
  }, [isAuthenticated, authRequired, redirectTo, router])

  // Fallback: if auth stays in a loading state for too long while unauthenticated,
  // send the user to the login page instead of spinning forever.
  useEffect(() => {
    if (!authRequired) return

    const timeout = setTimeout(() => {
      if (!isAuthenticated) {
        router.push(redirectTo)
      }
    }, 5000)

    return () => clearTimeout(timeout)
  }, [authRequired, isAuthenticated, redirectTo, router])

  if (authRequired && isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    )
  }

  if (authRequired && !isAuthenticated) {
    return null
  }

  return <>{children}</>
}