// ============================================================================
// ConHub Frontend Observability
// Provides structured logging, performance tracking, and trace correlation
// ============================================================================

/** Header name for trace ID propagation to backend */
export const TRACE_ID_HEADER = 'x-trace-id';
export const SPAN_ID_HEADER = 'x-span-id';
export const REQUEST_ID_HEADER = 'x-request-id';

export interface LogEntry {
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  context?: Record<string, unknown>;
  source: string;
  userId?: string;
  sessionId: string;
  traceId: string;
  spanId?: string;
  url: string;
  userAgent: string;
}

export interface PerformanceMetric {
  timestamp: string;
  metric: string;
  value: number;
  context?: Record<string, unknown>;
  url: string;
}

export interface UserAction {
  timestamp: string;
  action: string;
  element: string;
  context?: Record<string, unknown>;
  userId?: string;
  sessionId: string;
  url: string;
}

class ConHubLogger {
  private sessionId: string;
  private traceId: string;
  private userId?: string;
  private tenantId?: string;
  private logBuffer: LogEntry[] = [];
  private performanceBuffer: PerformanceMetric[] = [];
  private userActionBuffer: UserAction[] = [];
  private maxBufferSize = 100;
  private flushInterval = 30000; 
  private isProduction: boolean;
  private logLevel: string;
  private serviceName = 'frontend';

  constructor() {
    this.sessionId = this.generateSessionId();
    this.traceId = this.generateTraceId();
    this.isProduction = process.env.NODE_ENV === 'production';
    this.logLevel = process.env.NEXT_PUBLIC_LOG_LEVEL || 'info';
    
    setInterval(() => this.flush(), this.flushInterval);
    
    if (typeof window !== 'undefined') {
      window.addEventListener('beforeunload', () => this.flush());
      window.addEventListener('unload', () => this.flush());
      
      this.trackPagePerformance();
      this.setupUserActionTracking();
      this.setupErrorTracking();
      this.setupRouteTracking();
    }
  }

  /** Generate a new trace ID (called once per session or user flow) */
  private generateTraceId(): string {
    return `trace_${Date.now().toString(36)}_${Math.random().toString(36).substr(2, 12)}`;
  }

  /** Generate a span ID for a specific operation */
  private generateSpanId(): string {
    return Math.random().toString(36).substr(2, 16);
  }

  /** Get the current trace ID for correlation */
  public getTraceId(): string {
    return this.traceId;
  }

  /** Start a new trace (for a new user flow) */
  public newTrace(): string {
    this.traceId = this.generateTraceId();
    this.info('New trace started', { traceId: this.traceId }, 'trace');
    return this.traceId;
  }

  /** Get headers to propagate trace context to backend */
  public getTraceHeaders(): Record<string, string> {
    return {
      [TRACE_ID_HEADER]: this.traceId,
      [SPAN_ID_HEADER]: this.generateSpanId(),
      [REQUEST_ID_HEADER]: `req_${Date.now().toString(36)}_${Math.random().toString(36).substr(2, 8)}`,
    };
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private shouldLog(level: string): boolean {
    const levels = { debug: 0, info: 1, warn: 2, error: 3 };
    return levels[level as keyof typeof levels] >= levels[this.logLevel as keyof typeof levels];
  }

  public setUserId(userId: string) {
    this.userId = userId;
    this.info('User logged in', { userId }, 'auth');
  }

  public setTenantId(tenantId: string) {
    this.tenantId = tenantId;
    this.debug('Tenant set', { tenantId }, 'auth');
  }

  public clearUser() {
    const oldUserId = this.userId;
    this.userId = undefined;
    this.tenantId = undefined;
    this.info('User logged out', { previousUserId: oldUserId }, 'auth');
  }

  public debug(message: string, context?: Record<string, unknown>, source = 'app') {
    if (this.shouldLog('debug')) {
      this.log('debug', message, context, source);
    }
  }

  public info(message: string, context?: Record<string, unknown>, source = 'app') {
    if (this.shouldLog('info')) {
      this.log('info', message, context, source);
    }
  }

  public warn(message: string, context?: Record<string, unknown>, source = 'app') {
    if (this.shouldLog('warn')) {
      this.log('warn', message, context, source);
    }
  }

  public error(message: string, context?: Record<string, unknown>, source = 'app') {
    if (this.shouldLog('error')) {
      this.log('error', message, context, source);
    }
  }

  private log(level: LogEntry['level'], message: string, context?: Record<string, unknown>, source = 'app') {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      context: {
        ...context,
        service: this.serviceName,
        tenantId: this.tenantId,
      },
      source,
      userId: this.userId,
      sessionId: this.sessionId,
      traceId: this.traceId,
      url: typeof window !== 'undefined' ? window.location.href : '',
      userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : ''
    };

    
    if (!this.isProduction) {
      const consoleMethod = console[level] || console.log;
      consoleMethod(`[${level.toUpperCase()}] ${source}: ${message}`, context || '');
    }

    this.logBuffer.push(entry);
    this.checkBufferLimit();
  }

  public trackPerformance(metric: string, value: number, context?: Record<string, unknown>) {
    const perfMetric: PerformanceMetric = {
      timestamp: new Date().toISOString(),
      metric,
      value,
      context,
      url: typeof window !== 'undefined' ? window.location.href : ''
    };

    this.performanceBuffer.push(perfMetric);
    this.checkBufferLimit();

    
    if (metric.includes('duration') && value > 1000) {
      this.warn(`Slow operation detected: ${metric}`, { value, ...context });
    }
  }

  public trackUserAction(action: string, element: string, context?: Record<string, unknown>) {
    const userAction: UserAction = {
      timestamp: new Date().toISOString(),
      action,
      element,
      context,
      userId: this.userId,
      sessionId: this.sessionId,
      url: typeof window !== 'undefined' ? window.location.href : ''
    };

    this.userActionBuffer.push(userAction);
    this.checkBufferLimit();

    this.debug(`User action: ${action} on ${element}`, context, 'user-interaction');
  }

  private trackPagePerformance() {
    if (typeof window === 'undefined' || !window.performance) return;

    
    window.addEventListener('load', () => {
      setTimeout(() => {
        const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        if (navigation) {
          this.trackPerformance('page_load_duration', navigation.loadEventEnd - navigation.fetchStart);
          this.trackPerformance('dom_content_loaded', navigation.domContentLoadedEventEnd - navigation.fetchStart);
          this.trackPerformance('first_byte', navigation.responseStart - navigation.fetchStart);
        }

        
        if ('PerformanceObserver' in window) {
          const lcpObserver = new PerformanceObserver((list) => {
            const entries = list.getEntries();
            const lastEntry = entries[entries.length - 1];
            this.trackPerformance('largest_contentful_paint', lastEntry.startTime);
          });
          lcpObserver.observe({ type: 'largest-contentful-paint', buffered: true });

          
          const fidObserver = new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
              // 'first-input' entries are PerformanceEventTiming which include processingStart
              const processingStart = 'processingStart' in entry
                ? (entry as PerformanceEventTiming).processingStart
                : undefined;
              if (typeof processingStart === 'number') {
                this.trackPerformance('first_input_delay', processingStart - entry.startTime);
              }
            }
          });
          fidObserver.observe({ type: 'first-input', buffered: true });
        }
      }, 0);
    });
  }

  private setupUserActionTracking() {
    if (typeof window === 'undefined') return;

    
    document.addEventListener('click', (event) => {
      const target = event.target as HTMLElement;
      const elementInfo = this.getElementInfo(target);
      this.trackUserAction('click', elementInfo.selector, {
        text: elementInfo.text,
        position: { x: event.clientX, y: event.clientY }
      });
    });

    
    document.addEventListener('submit', (event) => {
      const target = event.target as HTMLFormElement;
      const elementInfo = this.getElementInfo(target);
      this.trackUserAction('form_submit', elementInfo.selector, {
        formId: target.id,
        formName: target.name
      });
    });

    
    document.addEventListener('focusin', (event) => {
      const target = event.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        const elementInfo = this.getElementInfo(target);
        this.trackUserAction('input_focus', elementInfo.selector, {
          inputType: (target as HTMLInputElement).type
        });
      }
    });
  }

  private setupErrorTracking() {
    if (typeof window === 'undefined') return;

    
    window.addEventListener('error', (event) => {
      this.error('JavaScript error', {
        message: event.message,
        filename: event.filename,
        line: event.lineno,
        column: event.colno,
        stack: event.error?.stack
      }, 'global-error');
    });

    
    window.addEventListener('unhandledrejection', (event) => {
      this.error('Unhandled promise rejection', {
        reason: event.reason,
        stack: event.reason?.stack
      }, 'promise-rejection');
    });
  }

  private getElementInfo(element: HTMLElement): { selector: string; text: string } {
    let selector = element.tagName.toLowerCase();

    if (element.id) {
      selector += `#${element.id}`;
    }

    // className can be a string (HTML) or SVGAnimatedString (SVG). Prefer classList when available.
    try {
      if (element.classList && element.classList.length > 0) {
        selector += `.${Array.from(element.classList).join('.')}`;
      } else if (typeof element.className === 'string' && element.className.trim()) {
        selector += `.${element.className.trim().split(/\s+/).join('.')}`;
      } else if (
        typeof (element as unknown as { className?: unknown }).className === 'object' &&
        (element as unknown as { className: { baseVal?: unknown } }).className &&
        typeof (element as unknown as { className: { baseVal?: unknown } }).className.baseVal === 'string'
      ) {
        const baseVal = (element as unknown as { className: { baseVal: string } }).className.baseVal.trim();
        if (baseVal) {
          selector += `.${baseVal.split(/\s+/).join('.')}`;
        }
      }
    } catch {
      // If any unexpected type occurs, skip class aggregation safely.
    }

    const text = element.textContent?.trim().substring(0, 50) || '';

    return { selector, text };
  }

  public trackAPICall(url: string, method: string, duration: number, status: number, error?: string) {
    const context = {
      method,
      status,
      duration,
      error
    };

    if (error) {
      this.error(`API call failed: ${method} ${url}`, context, 'api');
    } else if (status >= 400) {
      this.warn(`API call returned ${status}: ${method} ${url}`, context, 'api');
    } else {
      this.info(`API call completed: ${method} ${url}`, context, 'api');
    }

    this.trackPerformance('api_call_duration', duration, {
      url,
      method,
      status
    });
  }

  public trackRouteChange(from: string, to: string, duration?: number) {
    // Start a new trace for each major navigation
    this.newTrace();
    this.info('Route change', { from, to, duration }, 'navigation');
    
    if (duration) {
      this.trackPerformance('route_change_duration', duration, { from, to });
    }
  }

  /** Setup route change tracking for Next.js */
  private setupRouteTracking() {
    if (typeof window === 'undefined') return;
    
    // Track initial page view
    this.trackPageView(window.location.pathname);
    
    // Listen for popstate (browser back/forward)
    window.addEventListener('popstate', () => {
      this.trackPageView(window.location.pathname);
    });
  }

  /** Track a page view */
  public trackPageView(path: string, metadata?: Record<string, unknown>) {
    this.info('Page view', { path, ...metadata }, 'navigation');
    this.trackPerformance('page_view', 1, { path });
  }

  /** Track a button click with structured context */
  public trackButtonClick(buttonId: string, buttonText: string, metadata?: Record<string, unknown>) {
    this.trackUserAction('button_click', buttonId, {
      buttonText,
      ...metadata,
    });
  }

  /** Track a form submission */
  public trackFormSubmit(formId: string, formName: string, success: boolean, metadata?: Record<string, unknown>) {
    this.trackUserAction('form_submit', formId, {
      formName,
      success,
      ...metadata,
    });
  }

  /** Track a feature usage */
  public trackFeatureUsage(featureName: string, action: string, metadata?: Record<string, unknown>) {
    this.info('Feature usage', { feature: featureName, action, ...metadata }, 'feature');
  }

  /** Track a connector action */
  public trackConnectorAction(connector: string, action: 'connect' | 'disconnect' | 'sync' | 'error', metadata?: Record<string, unknown>) {
    this.info('Connector action', { connector, action, ...metadata }, 'connector');
  }

  /** Track a search query */
  public trackSearch(query: string, resultsCount: number, duration: number, metadata?: Record<string, unknown>) {
    this.info('Search executed', {
      queryLength: query.length,
      resultsCount,
      duration,
      ...metadata,
    }, 'search');
    this.trackPerformance('search_duration', duration, { resultsCount });
  }

  private checkBufferLimit() {
    if (this.logBuffer.length > this.maxBufferSize) {
      this.flush();
    }
  }

  private safeStringify(obj: unknown): string {
    const seen = new WeakSet();
    return JSON.stringify(obj, (key, value) => {
      if (typeof value === 'object' && value !== null) {
        if (seen.has(value)) {
          return '[Circular Reference]';
        }
        seen.add(value);
      }
      return value;
    });
  }

  public async flush() {
    if (this.logBuffer.length === 0 && this.performanceBuffer.length === 0 && this.userActionBuffer.length === 0) {
      return;
    }

    const payload = {
      logs: [...this.logBuffer],
      performance: [...this.performanceBuffer],
      userActions: [...this.userActionBuffer],
      sessionId: this.sessionId,
      timestamp: new Date().toISOString()
    };


    this.logBuffer = [];
    this.performanceBuffer = [];
    this.userActionBuffer = [];

    try {

      await fetch('/api/logs', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: this.safeStringify(payload)
      });
    } catch (error) {

      if (typeof localStorage !== 'undefined') {
        const stored = localStorage.getItem('conhub_logs') || '[]';
        const logs = JSON.parse(stored);
        logs.push(payload);


        if (logs.length > 10) {
          logs.splice(0, logs.length - 10);
        }

        localStorage.setItem('conhub_logs', this.safeStringify(logs));
      }
    }
  }


  public async retryFailedLogs() {
    if (typeof localStorage === 'undefined') return;

    const stored = localStorage.getItem('conhub_logs');
    if (!stored) return;

    const logs = JSON.parse(stored);

    for (const payload of logs) {
      try {
        await fetch('/api/logs', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: this.safeStringify(payload)
        });
      } catch (error) {

        break;
      }
    }


    localStorage.removeItem('conhub_logs');
  }
}


export const logger = new ConHubLogger();


export function useLogger() {
  return logger;
}


export async function apiCall(url: string, options: RequestInit = {}) {
  const startTime = performance.now();
  const method = options.method || 'GET';
  
  try {
    const response = await fetch(url, options);
    const duration = performance.now() - startTime;
    
    logger.trackAPICall(url, method, duration, response.status);
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return response;
  } catch (error) {
    const duration = performance.now() - startTime;
    logger.trackAPICall(url, method, duration, 0, error instanceof Error ? error.message : 'Unknown error');
    throw error;
  }
}

export default logger;
