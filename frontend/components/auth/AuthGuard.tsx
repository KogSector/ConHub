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

  useEffect(() => {
    if (!isLoading && authRequired && !isAuthenticated) {
      router.push(redirectTo)
    }
  }, [isAuthenticated, isLoading, authRequired, redirectTo, router])

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