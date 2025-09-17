'use client'

import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useState } from "react"
import { AuthProvider } from "@/contexts/auth-context"
import { isLoginEnabled } from "@/lib/feature-toggles"

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient())
  const loginEnabled = isLoginEnabled()

  return (
    <QueryClientProvider client={queryClient}>
      {loginEnabled ? (
        <AuthProvider>
          {children}
        </AuthProvider>
      ) : (
        // When login is disabled, just render children without auth
        <>{children}</>
      )}
    </QueryClientProvider>
  )
}