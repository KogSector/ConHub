'use client'

import { useCallback, useMemo, useRef, useEffect, useState } from 'react'
import { cn } from '@/lib/utils'


interface UseIntersectionObserverProps {
  ref: React.RefObject<Element>
  threshold?: number
  rootMargin?: string
  once?: boolean
}

function useIntersectionObserver({
  ref,
  threshold = 0.1,
  rootMargin = '0px',
  once = true
}: UseIntersectionObserverProps) {
  const [isIntersecting, setIsIntersecting] = useState(false)

  useEffect(() => {
    const element = ref.current
    if (!element) return

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsIntersecting(true)
          if (once) {
            observer.disconnect()
          }
        } else if (!once) {
          setIsIntersecting(false)
        }
      },
      { threshold, rootMargin }
    )

    observer.observe(element)

    return () => observer.disconnect()
  }, [ref, threshold, rootMargin, once])

  return isIntersecting
}


interface LazyComponentProps {
  children: React.ReactNode
  fallback?: React.ReactNode
  threshold?: number
  rootMargin?: string
  once?: boolean
  className?: string
  onIntersect?: () => void
}

export function LazyComponent({ 
  children, 
  fallback = <div className="loading-skeleton h-32 w-full rounded-lg" />,
  threshold = 0.1,
  rootMargin = '50px',
  once = true,
  className,
  onIntersect
}: LazyComponentProps) {
  const ref = useRef<HTMLDivElement>(null)
  const isIntersecting = useIntersectionObserver({
    ref,
    threshold,
    rootMargin,
    once
  })

  useEffect(() => {
    if (isIntersecting && onIntersect) {
      onIntersect()
    }
  }, [isIntersecting, onIntersect])

  return (
    <div 
      ref={ref} 
      className={cn(
        'lazy-load transition-all duration-300',
        isIntersecting && 'loaded',
        className
      )}
    >
      {isIntersecting ? children : fallback}
    </div>
  )
}


interface VirtualizedListProps<T> {
  items: T[]
  itemHeight: number | ((index: number) => number)
  renderItem: (item: T, index: number) => React.ReactNode
  overscan?: number
  className?: string
  onScrollEnd?: () => void
}

export function VirtualizedList<T>({
  items,
  itemHeight,
  renderItem,
  overscan = 5,
  className,
  onScrollEnd
}: VirtualizedListProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null)
  const scrollElementRef = useRef<HTMLDivElement>(null)

  
  const { visibleItems, totalHeight } = useMemo(() => {
    if (!containerRef.current) {
      return { visibleItems: [], totalHeight: 0 }
    }

    const containerHeight = containerRef.current.clientHeight
    const scrollTop = scrollElementRef.current?.scrollTop || 0
    
    let currentHeight = 0
    let startIndex = 0
    let endIndex = 0
    
    
    for (let i = 0; i < items.length; i++) {
      const height = typeof itemHeight === 'function' ? itemHeight(i) : itemHeight
      if (currentHeight + height > scrollTop) {
        startIndex = Math.max(0, i - overscan)
        break
      }
      currentHeight += height
    }
    
    
    currentHeight = 0
    for (let i = startIndex; i < items.length; i++) {
      const height = typeof itemHeight === 'function' ? itemHeight(i) : itemHeight
      currentHeight += height
      if (currentHeight > containerHeight + overscan * (typeof itemHeight === 'function' ? itemHeight(i) : itemHeight)) {
        endIndex = Math.min(items.length - 1, i + overscan)
        break
      }
    }
    
    const totalHeight = items.reduce((acc, _, index) => {
      return acc + (typeof itemHeight === 'function' ? itemHeight(index) : itemHeight)
    }, 0)
    
    const visibleItems = items.slice(startIndex, endIndex + 1).map((item, relativeIndex) => ({
      item,
      index: startIndex + relativeIndex
    }))
    
    return { visibleItems, totalHeight }
  }, [items, itemHeight, overscan])

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const { scrollTop, scrollHeight, clientHeight } = e.currentTarget
    
    
    if (scrollTop + clientHeight >= scrollHeight - 10 && onScrollEnd) {
      onScrollEnd()
    }
  }, [onScrollEnd])

  return (
    <div 
      ref={containerRef}
      className={cn('h-full overflow-auto', className)}
      onScroll={handleScroll}
    >
      <div 
        ref={scrollElementRef}
        style={{ height: totalHeight }}
        className="relative"
      >
        {visibleItems.map(({ item, index }) => (
          <div 
            key={index}
            className="absolute w-full"
            style={{
              top: typeof itemHeight === 'function' 
                ? items.slice(0, index).reduce((acc, _, i) => acc + itemHeight(i), 0)
                : index * itemHeight,
              height: typeof itemHeight === 'function' ? itemHeight(index) : itemHeight
            }}
          >
            {renderItem(item, index)}
          </div>
        ))}
      </div>
    </div>
  )
}


interface OptimizedImageProps {
  src: string
  alt: string
  width?: number
  height?: number
  className?: string
  priority?: boolean
  quality?: number
  placeholder?: 'blur' | 'empty'
  blurDataURL?: string
}

export function OptimizedImage({
  src,
  alt,
  width,
  height,
  className,
  priority = false,
  quality = 75,
  placeholder = 'empty',
  blurDataURL
}: OptimizedImageProps) {
  const [isLoaded, setIsLoaded] = useState(false)
  const [hasError, setHasError] = useState(false)
  const imgRef = useRef<HTMLImageElement>(null)

  
  const webpSrc = useMemo(() => {
    if (src.startsWith('data:') || src.includes('.svg')) return src
    return src.replace(/\.(jpg|jpeg|png)$/i, '.webp')
  }, [src])

  useEffect(() => {
    if (priority && imgRef.current) {
      imgRef.current.loading = 'eager'
    }
  }, [priority])

  return (
    <picture className={cn('block', className)}>
      <source srcSet={webpSrc} type="image/webp" />
      <img
        ref={imgRef}
        src={src}
        alt={alt}
        width={width}
        height={height}
        loading={priority ? 'eager' : 'lazy'}
        decoding="async"
        className={cn(
          'transition-opacity duration-300',
          !isLoaded && 'opacity-0',
          isLoaded && 'opacity-100',
          hasError && 'opacity-50'
        )}
        onLoad={() => setIsLoaded(true)}
        onError={() => setHasError(true)}
        style={{
          backgroundImage: placeholder === 'blur' && blurDataURL ? `url(${blurDataURL})` : undefined,
          backgroundSize: 'cover',
          backgroundPosition: 'center'
        }}
      />
    </picture>
  )
}


interface PerformantTableProps<T> {
  data: T[]
  columns: Array<{
    key: keyof T
    header: string
    render?: (value: any, row: T, index: number) => React.ReactNode
    width?: number
  }>
  rowHeight?: number
  maxHeight?: number
  onRowClick?: (row: T, index: number) => void
  loading?: boolean
  className?: string
}

export function PerformantTable<T extends Record<string, any>>({
  data,
  columns,
  rowHeight = 48,
  maxHeight = 400,
  onRowClick,
  loading = false,
  className
}: PerformantTableProps<T>) {
  const memoizedData = useMemo(() => data, [data])

  const renderRow = useCallback((item: T, index: number) => (
    <div 
      className={cn(
        'flex items-center border-b border-border hover:bg-muted/50 transition-colors cursor-pointer',
        'gpu-accelerated'
      )}
      onClick={() => onRowClick?.(item, index)}
      style={{ height: rowHeight }}
    >
      {columns.map((column) => (
        <div 
          key={String(column.key)}
          className="px-4 py-2 flex-1"
          style={{ width: column.width }}
        >
          {column.render 
            ? column.render(item[column.key], item, index)
            : String(item[column.key] || '')
          }
        </div>
      ))}
    </div>
  ), [columns, rowHeight, onRowClick])

  if (loading) {
    return (
      <div className="space-y-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="loading-skeleton h-12 w-full rounded" />
        ))}
      </div>
    )
  }

  return (
    <div className={cn('border border-border rounded-lg overflow-hidden', className)}>
      {}
      <div className="flex bg-muted font-medium text-sm">
        {columns.map((column) => (
          <div 
            key={String(column.key)}
            className="px-4 py-3 flex-1"
            style={{ width: column.width }}
          >
            {column.header}
          </div>
        ))}
      </div>
      
      {}
      <VirtualizedList
        items={memoizedData}
        itemHeight={rowHeight}
        renderItem={renderRow}
        className="max-h-96"
      />
    </div>
  )
}

export default {
  LazyComponent,
  VirtualizedList,
  OptimizedImage,
  PerformantTable
}