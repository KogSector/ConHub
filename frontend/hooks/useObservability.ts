/**
 * ConHub Frontend Observability Hooks
 * 
 * Provides React hooks for logging, tracking, and observability throughout the frontend.
 */

'use client';

import { useCallback, useEffect, useRef } from 'react';
import { usePathname } from 'next/navigation';
import logger from '../lib/logger';

/**
 * Hook to get the logger instance with tracking methods
 */
export function useLogger() {
  return logger;
}

/**
 * Hook to track page views automatically based on route changes
 */
export function usePageTracking() {
  const pathname = usePathname();
  const previousPathRef = useRef<string | null>(null);

  useEffect(() => {
    if (previousPathRef.current !== pathname) {
      const previousPath = previousPathRef.current;
      previousPathRef.current = pathname;
      
      if (previousPath) {
        logger.trackRouteChange(previousPath, pathname);
      } else {
        logger.trackPageView(pathname);
      }
    }
  }, [pathname]);
}

/**
 * Hook to track component mount/unmount for debugging
 */
export function useComponentTracking(componentName: string) {
  useEffect(() => {
    logger.debug(`Component mounted: ${componentName}`, { component: componentName }, 'component');
    
    return () => {
      logger.debug(`Component unmounted: ${componentName}`, { component: componentName }, 'component');
    };
  }, [componentName]);
}

/**
 * Hook to track button clicks with logging
 */
export function useButtonTracking() {
  const trackClick = useCallback((buttonId: string, buttonText: string, metadata?: Record<string, unknown>) => {
    logger.trackButtonClick(buttonId, buttonText, metadata);
  }, []);

  return trackClick;
}

/**
 * Hook to track form submissions with logging
 */
export function useFormTracking(formId: string, formName: string) {
  const trackSubmit = useCallback((success: boolean, metadata?: Record<string, unknown>) => {
    logger.trackFormSubmit(formId, formName, success, metadata);
  }, [formId, formName]);

  return trackSubmit;
}

/**
 * Hook to track async operations with timing and error logging
 */
export function useAsyncTracking() {
  const trackAsync = useCallback(async <T>(
    operationName: string,
    operation: () => Promise<T>,
    metadata?: Record<string, unknown>
  ): Promise<T> => {
    const startTime = performance.now();
    logger.debug(`Operation started: ${operationName}`, { operation: operationName, ...metadata }, 'async');
    
    try {
      const result = await operation();
      const duration = performance.now() - startTime;
      
      logger.info(`Operation completed: ${operationName}`, {
        operation: operationName,
        duration,
        success: true,
        ...metadata,
      }, 'async');
      
      logger.trackPerformance(`operation_${operationName}`, duration, metadata);
      
      return result;
    } catch (error) {
      const duration = performance.now() - startTime;
      
      logger.error(`Operation failed: ${operationName}`, {
        operation: operationName,
        duration,
        error: error instanceof Error ? error.message : String(error),
        ...metadata,
      }, 'async');
      
      throw error;
    }
  }, []);

  return trackAsync;
}

/**
 * Hook to track feature usage
 */
export function useFeatureTracking() {
  const trackFeature = useCallback((featureName: string, action: string, metadata?: Record<string, unknown>) => {
    logger.trackFeatureUsage(featureName, action, metadata);
  }, []);

  return trackFeature;
}

/**
 * Hook to track connector actions (GitHub, Dropbox, etc.)
 */
export function useConnectorTracking() {
  const trackConnector = useCallback((
    connector: string,
    action: 'connect' | 'disconnect' | 'sync' | 'error',
    metadata?: Record<string, unknown>
  ) => {
    logger.trackConnectorAction(connector, action, metadata);
  }, []);

  return trackConnector;
}

/**
 * Hook to track search operations
 */
export function useSearchTracking() {
  const trackSearch = useCallback((query: string, resultsCount: number, duration: number, metadata?: Record<string, unknown>) => {
    logger.trackSearch(query, resultsCount, duration, metadata);
  }, []);

  return trackSearch;
}

/**
 * Hook to set user context for logging
 */
export function useUserContext(userId?: string, tenantId?: string) {
  useEffect(() => {
    if (userId) {
      logger.setUserId(userId);
    }
    if (tenantId) {
      logger.setTenantId(tenantId);
    }
    
    return () => {
      if (userId) {
        logger.clearUser();
      }
    };
  }, [userId, tenantId]);
}

/**
 * Hook to create a traced fetch function that includes trace headers
 */
export function useTracedFetch() {
  const tracedFetch = useCallback(async (url: string, options: RequestInit = {}): Promise<Response> => {
    const traceHeaders = logger.getTraceHeaders();
    const startTime = performance.now();
    const method = options.method || 'GET';
    
    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          ...options.headers,
          ...traceHeaders,
        },
      });
      
      const duration = performance.now() - startTime;
      logger.trackAPICall(url, method, duration, response.status);
      
      return response;
    } catch (error) {
      const duration = performance.now() - startTime;
      logger.trackAPICall(url, method, duration, 0, error instanceof Error ? error.message : 'Unknown error');
      throw error;
    }
  }, []);

  return tracedFetch;
}

/**
 * Get the current trace ID (useful for displaying in error messages)
 */
export function getTraceId(): string {
  return logger.getTraceId();
}

/**
 * Get trace headers for manual fetch calls
 */
export function getTraceHeaders(): Record<string, string> {
  return logger.getTraceHeaders();
}
