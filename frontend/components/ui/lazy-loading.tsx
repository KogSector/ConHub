import dynamic from 'next/dynamic'
import { ComponentType } from 'react'

interface LoadingSkeletonProps {
  height?: string
  width?: string
  className?: string
}

export function LoadingSkeleton({ 
  height = "h-32", 
  width = "w-full", 
  className = "" 
}: LoadingSkeletonProps) {
  return (
    <div className={`${height} ${width} bg-muted animate-pulse rounded-lg ${className}`}>
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground/50 text-sm">Loading...</div>
      </div>
    </div>
  )
}

// Utility function for lazy loading components with custom loading states
export function lazyLoad<T extends ComponentType<any>>(
  importFunc: () => Promise<{ default: T }>,
  loadingComponent?: () => JSX.Element,
  options: { ssr?: boolean } = {}
) {
  return dynamic(importFunc, {
    loading: loadingComponent || (() => <LoadingSkeleton />),
    ssr: options.ssr ?? true
  })
}

// Pre-built lazy components for common UI elements
export const LazySocialConnections = lazyLoad(
  () => import('@/components/social/social-connections'),
  () => <LoadingSkeleton height="h-32" className="border" />
)