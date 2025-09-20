'use client'

import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useState } from "react"
import { AuthProvider } from "@/contexts/auth-context"
import { LoggingProvider } from "@/components/providers/logging-provider"
import { useAuth } from "@/hooks/use-auth"

function LoggingWrapper({ children }: { children: React.ReactNode }) {
  const { user } = useAuth()
  
  return (
    <LoggingProvider userId={user?.id}>
      {children}
    </LoggingProvider>
  )
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient())

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <LoggingWrapper>
          {children}
        </LoggingWrapper>
      </AuthProvider>
    </QueryClientProvider>
  )
}