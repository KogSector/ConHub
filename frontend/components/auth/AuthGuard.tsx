'use client'

import { useAuth } from "@/hooks/use-auth"
import { isLoginEnabled } from "@/lib/feature-toggles"
import { ReactNode } from "react"

interface AuthGuardProps {
  children: ReactNode
  fallback?: ReactNode
}

export const AuthGuard = ({ children, fallback }: AuthGuardProps) => {
  const { isAuthenticated, isLoading } = useAuth()
  const loginEnabled = isLoginEnabled()

  // If login is disabled, always allow access
  if (!loginEnabled) {
    return <>{children}</>
  }

  // Show loading state
  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p>Loading...</p>
        </div>
      </div>
    )
  }

  // Show fallback or redirect if not authenticated
  if (!isAuthenticated) {
    return fallback || (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <h2 className="text-xl font-semibold mb-4">Authentication Required</h2>
          <p>Please sign in to access this page.</p>
        </div>
      </div>
    )
  }

  return <>{children}</>
}