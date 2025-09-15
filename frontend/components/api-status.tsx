'use client';

import React, { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api';
import { AlertCircle, CheckCircle, Clock } from 'lucide-react';

interface ApiStatusProps {
  showDetails?: boolean;
  className?: string;
}

export function ApiStatus({ showDetails = false, className = '' }: ApiStatusProps) {
  const [status, setStatus] = useState<'checking' | 'connected' | 'disconnected'>('checking');
  const [lastCheck, setLastCheck] = useState<Date | null>(null);

  const checkApiStatus = async () => {
    setStatus('checking');
    try {
      const isHealthy = await apiClient.checkBackendHealth();
      setStatus(isHealthy ? 'connected' : 'disconnected');
      setLastCheck(new Date());
    } catch (error) {
      console.error('API status check failed:', error);
      setStatus('disconnected');
      setLastCheck(new Date());
    }
  };

  useEffect(() => {
    checkApiStatus();
    
    // Check every 30 seconds
    const interval = setInterval(checkApiStatus, 30000);
    
    return () => clearInterval(interval);
  }, []);

  const getStatusIcon = () => {
    switch (status) {
      case 'checking':
        return <Clock className="w-4 h-4 text-yellow-500 animate-spin" />;
      case 'connected':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'disconnected':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'checking':
        return 'Checking...';
      case 'connected':
        return 'Connected';
      case 'disconnected':
        return 'Disconnected';
    }
  };

  const getStatusColor = () => {
    switch (status) {
      case 'checking':
        return 'text-yellow-600';
      case 'connected':
        return 'text-green-600';
      case 'disconnected':
        return 'text-red-600';
    }
  };

  if (!showDetails) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        {getStatusIcon()}
        <span className={`text-sm ${getStatusColor()}`}>
          {getStatusText()}
        </span>
      </div>
    );
  }

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg border p-4 ${className}`}>
      <div className="flex items-center justify-between mb-2">
        <h3 className="font-medium text-gray-900 dark:text-white">API Status</h3>
        <button
          onClick={checkApiStatus}
          className="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400"
          disabled={status === 'checking'}
        >
          Refresh
        </button>
      </div>
      
      <div className="flex items-center gap-2 mb-2">
        {getStatusIcon()}
        <span className={`font-medium ${getStatusColor()}`}>
          {getStatusText()}
        </span>
      </div>

      {lastCheck && (
        <p className="text-xs text-gray-500 dark:text-gray-400">
          Last checked: {lastCheck.toLocaleTimeString()}
        </p>
      )}

      {status === 'disconnected' && (
        <div className="mt-3 p-2 bg-red-50 dark:bg-red-900/20 rounded border border-red-200 dark:border-red-800">
          <p className="text-sm text-red-800 dark:text-red-200">
            Cannot connect to backend API. Please ensure the backend server is running at{' '}
            <code className="font-mono">http://localhost:3001</code>
          </p>
        </div>
      )}
    </div>
  );
}