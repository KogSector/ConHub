import { API_CONFIG } from './config';

export interface DocumentRecord {
  id: string;
  user_id: string;
  name: string;
  doc_type: string;
  source: string;
  size: string;
  tags: string[];
  created_at: string;
  updated_at: string;
  status: string;
}

export interface ApiResponse<T = any> {
  success: boolean;
  message: string;
  data?: T;
  error?: string;
}

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl = API_CONFIG.baseUrl) {
    this.baseUrl = baseUrl;
    // Log configuration in development
    if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
      console.log('API Client initialized with baseUrl:', this.baseUrl);
    }
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    let data;
    try {
      data = await response.json();
    } catch (error) {
      throw new Error(`Failed to parse response: ${response.statusText}`);
    }

    if (!response.ok) {
      const errorMessage = data?.error || data?.message || `HTTP error! status: ${response.status}`;
      throw new Error(errorMessage);
    }

    return data;
  }

  async get<T = any>(endpoint: string): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API GET ${endpoint} failed:`, error);
      throw error;
    }
  }

  async post<T = any>(endpoint: string, data: any): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API POST ${endpoint} failed:`, error);
      throw error;
    }
  }

  async put<T = any>(endpoint: string, data: any): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API PUT ${endpoint} failed:`, error);
      throw error;
    }
  }

  async delete<T = any>(endpoint: string): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API DELETE ${endpoint} failed:`, error);
      throw error;
    }
  }
  async health(): Promise<ApiResponse> {
    return this.get('/health');
  }

  // URL-specific methods
  async createUrl(data: {
    url: string;
    title?: string;
    description?: string;
    tags?: string[];
  }): Promise<ApiResponse> {
    return this.post('/api/urls', data);
  }

  async getUrls(): Promise<ApiResponse> {
    return this.get('/api/urls');
  }

  async deleteUrl(id: string): Promise<ApiResponse> {
    return this.delete(`/api/urls/${id}`);
  }

  // Document-specific methods
  async createDocument(data: {
    name: string;
    source: string;
    doc_type: string;
    size?: string;
    tags?: string[];
  }): Promise<ApiResponse<DocumentRecord>> {
    return this.post('/api/documents', data);
  }

  async getDocuments(search?: string): Promise<ApiResponse<{data: DocumentRecord[], total: number}>> {
    const params = search ? `?search=${encodeURIComponent(search)}` : '';
    return this.get(`/api/documents${params}`);
  }

  async deleteDocument(id: string): Promise<ApiResponse> {
    return this.delete(`/api/documents/${id}`);
  }

  async getDocumentAnalytics(): Promise<ApiResponse> {
    return this.get('/api/documents/analytics');
  }

  // Health check method to test backend connectivity
  async checkBackendHealth(): Promise<boolean> {
    try {
      const response = await this.health();
      return response.success || response.status === 'ok';
    } catch (error) {
      console.error('Backend health check failed:', error);
      return false;
    }
  }
}

export const apiClient = new ApiClient();
