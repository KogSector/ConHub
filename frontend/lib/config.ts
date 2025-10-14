

export const API_CONFIG = {
  baseUrl: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001',
  timeout: 10000,
  retries: 3,
  retryDelay: 1000,
} as const;

export const ENDPOINTS = {
  health: '/health',
  urls: '/api/urls',
  documents: '/api/documents',
  settings: '/api/settings',
} as const;


if (typeof window !== 'undefined') {
  const isValidUrl = (url: string) => {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  };

  if (!isValidUrl(API_CONFIG.baseUrl)) {
    console.error('Invalid API base URL:', API_CONFIG.baseUrl);
  }

  
  if (process.env.NODE_ENV === 'development') {
    console.log('ConHub API Configuration:', {
      baseUrl: API_CONFIG.baseUrl,
      timeout: API_CONFIG.timeout,
      environment: process.env.NODE_ENV,
      buildTime: new Date().toISOString(),
    });
  }
}

export default API_CONFIG;