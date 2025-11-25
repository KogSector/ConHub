'use client'

import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useState } from "react"
// 👇 1. Import Auth0 hook instead of old auth
import { useAuth0 } from "@auth0/auth0-react" 
import { LoggingProvider } from "@/components/providers/LoggingProvider"

function LoggingWrapper({ children }: { children: React.ReactNode }) {
  // 👇 2. Use the Auth0 hook to get the user ID
  const { user } = useAuth0()
  
  return (
    // Auth0 uses 'sub' as the unique ID. We pass that to your logger.
    <LoggingProvider userId={user?.sub}>
      {children}
    </LoggingProvider>
  )
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient())

  return (
    <QueryClientProvider client={queryClient}>
      {/* 👇 3. REMOVED <AuthProvider> wrapper. 
          Auth0Provider in layout.tsx handles auth now. */}
      <LoggingWrapper>
        {children}
      </LoggingWrapper>
    </QueryClientProvider>
  )
}