'use client'

import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useState } from "react"
import Auth0ProviderWrapper from "@/components/auth/Auth0ProviderWrapper"

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient())

  return (
    <QueryClientProvider client={queryClient}>
      <Auth0ProviderWrapper>
        {children}
      </Auth0ProviderWrapper>
    </QueryClientProvider>
  )
}