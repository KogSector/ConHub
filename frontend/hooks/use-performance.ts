'use client'

import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

/**
 * Advanced performance hooks for React optimization
 */

/**
 * Debounced value hook with cleanup
 */
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value)
    }, delay)

    return () => {
      clearTimeout(handler)
    }
  }, [value, delay])

  return debouncedValue
}

/**
 * Throttled callback hook for performance-critical operations
 */
export function useThrottle<T extends (...args: any[]) => any>(
  callback: T,
  delay: number
): T {
  const lastRun = useRef(Date.now())

  return useCallback(
    ((...args) => {
      if (Date.now() - lastRun.current >= delay) {
        callback(...args)
        lastRun.current = Date.now()
      }
    }) as T,
    [callback, delay]
  )
}

/**
 * Memoized async data fetching with caching
 */
interface UseAsyncDataOptions<T> {
  initialData?: T
  cacheKey?: string
  cacheTTL?: number
  retryAttempts?: number
  retryDelay?: number
}

export function useAsyncData<T>(
  asyncFunction: () => Promise<T>,
  dependencies: React.DependencyList = [],
  options: UseAsyncDataOptions<T> = {}
) {
  const {
    initialData,
    cacheKey,
    cacheTTL = 300000, // 5 minutes
    retryAttempts = 3,
    retryDelay = 1000
  } = options

  const [data, setData] = useState<T | undefined>(initialData)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const retryCount = useRef(0)
  const cache = useRef(new Map<string, { data: T; timestamp: number }>())

  const fetchData = useCallback(async () => {
    // Check cache first
    if (cacheKey && cache.current.has(cacheKey)) {
      const cached = cache.current.get(cacheKey)!
      if (Date.now() - cached.timestamp < cacheTTL) {
        setData(cached.data)
        return
      }
    }

    setLoading(true)
    setError(null)

    const attemptFetch = async (attempt: number): Promise<void> => {
      try {
        const result = await asyncFunction()
        setData(result)
        
        // Cache the result
        if (cacheKey) {
          cache.current.set(cacheKey, { data: result, timestamp: Date.now() })
        }
        
        retryCount.current = 0
      } catch (err) {
        if (attempt < retryAttempts) {
          setTimeout(() => attemptFetch(attempt + 1), retryDelay * attempt)
        } else {
          setError(err instanceof Error ? err : new Error('Unknown error'))
        }
      }
    }

    await attemptFetch(1)
    setLoading(false)
  }, dependencies)

  useEffect(() => {
    fetchData()
  }, dependencies)

  return { data, loading, error, refetch: fetchData }
}

/**
 * Performance-optimized infinite scroll hook
 */
interface UseInfiniteScrollOptions {
  threshold?: number
  rootMargin?: string
  hasMore?: boolean
  loading?: boolean
}

export function useInfiniteScroll(
  callback: () => void,
  options: UseInfiniteScrollOptions = {}
) {
  const { threshold = 0.1, rootMargin = '100px', hasMore = true, loading = false } = options
  const [isFetching, setIsFetching] = useState(false)
  const sentinelRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const sentinel = sentinelRef.current
    if (!sentinel || !hasMore || loading) return

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !isFetching) {
          setIsFetching(true)
          callback()
          setTimeout(() => setIsFetching(false), 100) // Prevent rapid firing
        }
      },
      { threshold, rootMargin }
    )

    observer.observe(sentinel)

    return () => observer.disconnect()
  }, [callback, threshold, rootMargin, hasMore, loading, isFetching])

  return { sentinelRef, isFetching }
}

/**
 * Memory-efficient list state management for large datasets
 */
export function useVirtualizedState<T>(
  items: T[],
  windowSize: number = 50
) {
  const [visibleRange, setVisibleRange] = useState({ start: 0, end: windowSize })
  const [scrollTop, setScrollTop] = useState(0)

  const visibleItems = useMemo(() => {
    return items.slice(visibleRange.start, visibleRange.end)
  }, [items, visibleRange])

  const updateVisibleRange = useCallback((containerHeight: number, itemHeight: number, scrollTop: number) => {
    const start = Math.floor(scrollTop / itemHeight)
    const visibleCount = Math.ceil(containerHeight / itemHeight)
    const end = Math.min(start + visibleCount + 10, items.length) // 10 item buffer

    setVisibleRange({ start: Math.max(0, start - 5), end }) // 5 item pre-buffer
    setScrollTop(scrollTop)
  }, [items.length])

  return {
    visibleItems,
    visibleRange,
    updateVisibleRange,
    totalHeight: items.length * 50, // Default item height
    scrollTop
  }
}

/**
 * Performance monitoring hook
 */
export function usePerformanceMonitor(componentName: string) {
  const renderCount = useRef(0)
  const startTime = useRef(performance.now())

  useEffect(() => {
    renderCount.current += 1
    const endTime = performance.now()
    const renderTime = endTime - startTime.current

    if (process.env.NODE_ENV === 'development') {
      console.log(`[Performance] ${componentName} - Render #${renderCount.current}, Time: ${renderTime.toFixed(2)}ms`)
    }

    startTime.current = performance.now()
  })

  return { renderCount: renderCount.current }
}

/**
 * Optimized state updates with batching
 */
export function useBatchedState<T>(initialState: T) {
  const [state, setState] = useState(initialState)
  const pendingUpdates = useRef<Partial<T>[]>([])
  const timeoutRef = useRef<NodeJS.Timeout>()

  const batchedSetState = useCallback((update: Partial<T> | ((prev: T) => Partial<T>)) => {
    const updateObj = typeof update === 'function' ? update(state) : update
    pendingUpdates.current.push(updateObj)

    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
    }

    timeoutRef.current = setTimeout(() => {
      setState(prevState => {
        const finalUpdate = pendingUpdates.current.reduce(
          (acc, update) => ({ ...acc, ...update }),
          {}
        )
        pendingUpdates.current = []
        return { ...prevState, ...finalUpdate }
      })
    }, 16) // Next frame
  }, [state])

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return [state, batchedSetState] as const
}

/**
 * Web Worker hook for CPU-intensive operations
 */
export function useWebWorker<T, R>(
  workerFunction: (data: T) => R,
  dependencies: React.DependencyList = []
) {
  const [result, setResult] = useState<R | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const workerRef = useRef<Worker | null>(null)

  const executeInWorker = useCallback(async (data: T): Promise<R> => {
    setLoading(true)
    setError(null)

    return new Promise((resolve, reject) => {
      // Create worker from function
      const workerScript = `
        self.onmessage = function(e) {
          try {
            const result = (${workerFunction.toString()})(e.data);
            self.postMessage({ success: true, result });
          } catch (error) {
            self.postMessage({ success: false, error: error.message });
          }
        };
      `

      const blob = new Blob([workerScript], { type: 'application/javascript' })
      const workerUrl = URL.createObjectURL(blob)
      const worker = new Worker(workerUrl)

      worker.onmessage = (e) => {
        const { success, result, error } = e.data
        setLoading(false)

        if (success) {
          setResult(result)
          resolve(result)
        } else {
          const err = new Error(error)
          setError(err)
          reject(err)
        }

        worker.terminate()
        URL.revokeObjectURL(workerUrl)
      }

      worker.onerror = (err) => {
        setLoading(false)
        setError(new Error('Worker error'))
        reject(err)
        worker.terminate()
      }

      worker.postMessage(data)
      workerRef.current = worker
    })
  }, dependencies)

  useEffect(() => {
    return () => {
      if (workerRef.current) {
        workerRef.current.terminate()
      }
    }
  }, [])

  return { executeInWorker, result, loading, error }
}