'use client';

import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { logger } from '@/lib/logger';
import { useRouter } from 'next/navigation';
import { usePathname } from 'next/navigation';

interface LoggingContextType {
  logger: typeof logger;
}

const LoggingContext = createContext<LoggingContextType | undefined>(undefined);

interface LoggingProviderProps {
  children: ReactNode;
  userId?: string;
}

export function LoggingProvider({ children, userId }: LoggingProviderProps) {
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    
    if (userId) {
      logger.setUserId(userId);
    }

    
    logger.retryFailedLogs();

    
    logger.info('Application initialized', {
      url: window.location.href,
      userAgent: navigator.userAgent,
      viewport: {
        width: window.innerWidth,
        height: window.innerHeight
      },
      timestamp: new Date().toISOString()
    }, 'app-init');

    
    let previousPath = pathname;
    const routeChangeStart = performance.now();

    const handleRouteChange = () => {
      const duration = performance.now() - routeChangeStart;
      logger.trackRouteChange(previousPath, pathname, duration);
      previousPath = pathname;
    };

    
    handleRouteChange();

    
    const handleVisibilityChange = () => {
      if (document.hidden) {
        logger.info('Page became hidden');
        logger.flush(); 
      } else {
        logger.info('Page became visible');
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    
    const handleOnline = () => {
      logger.info('Connection restored');
      logger.retryFailedLogs();
    };

    const handleOffline = () => {
      logger.warn('Connection lost');
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [userId, pathname]);

  const contextValue: LoggingContextType = {
    logger
  };

  return (
    <LoggingContext.Provider value={contextValue}>
      {children}
    </LoggingContext.Provider>
  );
}

export function useLogging() {
  const context = useContext(LoggingContext);
  if (context === undefined) {
    throw new Error('useLogging must be used within a LoggingProvider');
  }
  return context;
}


export function withLogging<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  componentName?: string
) {
  const ComponentWithLogging = (props: P) => {
    const { logger } = useLogging();
    const displayName = componentName || WrappedComponent.displayName || WrappedComponent.name || 'Component';

    useEffect(() => {
      logger.debug(`Component mounted: ${displayName}`, {}, 'component-lifecycle');

      return () => {
        logger.debug(`Component unmounted: ${displayName}`, {}, 'component-lifecycle');
      };
    }, [logger, displayName]);

    return <WrappedComponent {...props} />;
  };

  ComponentWithLogging.displayName = `withLogging(${componentName || WrappedComponent.displayName || WrappedComponent.name})`;

  return ComponentWithLogging;
}


export function useFormTracking(formName: string) {
  const { logger } = useLogging();

  const trackFormStart = (fields: string[]) => {
    logger.trackUserAction('form_start', formName, { fields });
  };

  const trackFormSubmit = (data: Record<string, any>, success: boolean, error?: string) => {
    logger.trackUserAction('form_submit', formName, {
      fieldCount: Object.keys(data).length,
      success,
      error
    });

    if (success) {
      logger.info(`Form submitted successfully: ${formName}`, { fieldCount: Object.keys(data).length });
    } else {
      logger.error(`Form submission failed: ${formName}`, { error });
    }
  };

  const trackFieldError = (fieldName: string, error: string) => {
    logger.warn(`Form field error: ${fieldName}`, { formName, error }, 'form-validation');
  };

  return {
    trackFormStart,
    trackFormSubmit,
    trackFieldError
  };
}


export function useSearchTracking() {
  const { logger } = useLogging();

  const trackSearch = (query: string, filters: Record<string, any>, resultsCount: number, duration: number) => {
    logger.info('Search performed', {
      query: query.substring(0, 100), 
      filters,
      resultsCount,
      duration
    }, 'search');

    logger.trackPerformance('search_duration', duration, {
      queryLength: query.length,
      filterCount: Object.keys(filters).length,
      resultsCount
    });

    logger.trackUserAction('search', 'search-input', {
      queryLength: query.length,
      resultsCount
    });
  };

  const trackSearchFilter = (filterType: string, filterValue: string) => {
    logger.trackUserAction('search_filter', 'filter-control', {
      filterType,
      filterValue
    });
  };

  const trackSearchResult = (resultIndex: number, resultType: string, resultId: string) => {
    logger.trackUserAction('search_result_click', 'search-result', {
      resultIndex,
      resultType,
      resultId
    });
  };

  return {
    trackSearch,
    trackSearchFilter,
    trackSearchResult
  };
}


export function useAITracking() {
  const { logger } = useLogging();

  const trackAIQuery = (query: string, agentType: string, duration: number, success: boolean, error?: string) => {
    logger.info('AI query performed', {
      queryLength: query.length,
      agentType,
      duration,
      success,
      error
    }, 'ai-interaction');

    logger.trackPerformance('ai_query_duration', duration, {
      agentType,
      queryLength: query.length,
      success
    });

    logger.trackUserAction('ai_query', 'ai-chat', {
      agentType,
      queryLength: query.length,
      success
    });
  };

  const trackAIResponse = (responseLength: number, useful: boolean, agentType: string) => {
    logger.trackUserAction('ai_response_feedback', 'feedback-button', {
      responseLength,
      useful,
      agentType
    });
  };

  return {
    trackAIQuery,
    trackAIResponse
  };
}