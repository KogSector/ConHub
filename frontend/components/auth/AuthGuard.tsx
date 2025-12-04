'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { useAuth } from '@/hooks/use-auth'

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

  useEffect(() => {
    if (requireAuth && !isLoading && !isAuthenticated) {
      router.push(redirectTo)
    }
  }, [isAuthenticated, isLoading, requireAuth, redirectTo, router])

  // Fallback: if auth stays in a loading state for too long while unauthenticated,
  // send the user to the login page instead of spinning forever.
  useEffect(() => {
    if (!requireAuth) return

    const timeout = setTimeout(() => {
      if (!isAuthenticated) {
        router.push(redirectTo)
      }
    }, 5000)

    return () => clearTimeout(timeout)
  }, [requireAuth, isAuthenticated, redirectTo, router])

  if (requireAuth && isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    )
  }

  if (requireAuth && !isAuthenticated) {
    return null
  }

  return <>{children}</>
}