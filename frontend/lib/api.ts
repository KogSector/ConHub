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

export interface AgentRecord {
  id: string;
  user_id: string;
  name: string;
  agent_type: string;
  endpoint?: string;
  api_key: string;
  permissions: string[];
  status: 'Connected' | 'Pending' | 'Error' | 'Inactive';
  config: AgentConfig;
  created_at: string;
  updated_at: string;
  last_used?: string;
  usage_stats: AgentUsageStats;
}

export interface AgentConfig {
  model?: string;
  temperature?: number;
  max_tokens?: number;
  timeout?: number;
  custom_instructions?: string;
}

export interface AgentUsageStats {
  total_requests: number;
  total_tokens: number;
  avg_response_time?: number;
  last_error?: string;
}

export interface CreateAgentRequest {
  name: string;
  agent_type: string;
  endpoint?: string;
  api_key: string;
  permissions: string[];
  config: AgentConfig;
}

export interface UpdateAgentRequest {
  name?: string;
  endpoint?: string;
  api_key?: string;
  permissions?: string[];
  config?: AgentConfig;
  status?: AgentRecord['status'];
}

export interface AgentInvokeRequest {
  message: string;
  context_type?: string;
  include_history?: boolean;
}

export interface AgentInvokeResponse {
  response: string;
  usage: {
    tokens_used: number;
    response_time_ms: number;
  };
  context_used: string[];
}

export interface ApiResponse<T = unknown> {
  success: boolean;
  message: string;
  data?: T;
  error?: string;
}

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl = API_CONFIG.baseUrl) {
    this.baseUrl = baseUrl;
    
    if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
      console.log('API Client initialized with baseUrl:', this.baseUrl);
    }
  }

  private async handleGraphQLResponse<T>(response: Response): Promise<T> {
    let payload: unknown;
    try {
      payload = await response.json();
    } catch (error) {
      throw new Error(`Failed to parse GraphQL response: ${response.statusText}`);
    }
    const obj = payload as { data?: T; errors?: { message?: string }[] };
    if (!response.ok || obj.errors?.length) {
      const msg = obj.errors?.map(e => e.message || 'GraphQL error').join('\n') || `HTTP error! status: ${response.status}`;
      throw new Error(msg);
    }
    if (!obj.data) {
      throw new Error('GraphQL response missing data');
    }
    return obj.data;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    let data: unknown;
    try {
      data = await response.json();
    } catch (error) {
      throw new Error(`Failed to parse response: ${response.statusText}`);
    }

    if (!response.ok) {
      const dataObj = typeof data === 'object' && data !== null ? data as Record<string, unknown> : {}
      const errorMessage = typeof dataObj['error'] === 'string' ? dataObj['error'] : (typeof dataObj['message'] === 'string' ? dataObj['message'] : `HTTP error! status: ${response.status}`)
      throw new Error(errorMessage)
    }

    return data as T;
  }

  async get<T = unknown>(endpoint: string, headers: Record<string, string> = {}): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API GET ${endpoint} failed:`, error);
      throw error;
    }
  }

  async graphql<T = unknown>(query: string, variables: Record<string, unknown> = {}, headers: Record<string, string> = {}): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}/api/graphql`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        body: JSON.stringify({ query, variables }),
        credentials: 'include',
      });
      return this.handleGraphQLResponse<T>(response);
    } catch (error) {
      console.error('GraphQL request failed:', error);
      throw error;
    }
  }
  async post<T = unknown>(endpoint: string, data: unknown, headers: Record<string, string> = {}): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        body: JSON.stringify(data),
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API POST ${endpoint} failed:`, error);
      throw error;
    }
  }
  async put<T = unknown>(endpoint: string, data: unknown, headers: Record<string, string> = {}): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        body: JSON.stringify(data),
      });
      return this.handleResponse<T>(response);
    } catch (error) {
      console.error(`API PUT ${endpoint} failed:`, error);
      throw error;
    }
  }
  async delete<T = unknown>(endpoint: string, headers: Record<string, string> = {}): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
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

  
  async checkBackendHealth(): Promise<boolean> {
    try {
      const response = await this.health();
      return response.success;
    } catch (error) {
      console.error('Backend health check failed:', error);
      return false;
    }
  }

  
  async getAgents(): Promise<ApiResponse<AgentRecord[]>> {
    return this.get('/api/agents');
  }

  async getAgent(id: string): Promise<ApiResponse<AgentRecord>> {
    return this.get(`/api/agents/${id}`);
  }

  async createAgent(data: CreateAgentRequest): Promise<ApiResponse<AgentRecord>> {
    return this.post('/api/agents', data);
  }

  async updateAgent(id: string, data: UpdateAgentRequest): Promise<ApiResponse<AgentRecord>> {
    return this.put(`/api/agents/${id}`, data);
  }

  async deleteAgent(id: string): Promise<ApiResponse> {
    return this.delete(`/api/agents/${id}`);
  }

  async getAgentContext(id: string): Promise<ApiResponse> {
    return this.get(`/api/agents/${id}/context`);
  }

  async invokeAgent(id: string, data: AgentInvokeRequest): Promise<ApiResponse<AgentInvokeResponse>> {
    return this.post(`/api/agents/${id}/invoke`, data);
  }

  async testAgent(id: string): Promise<ApiResponse<{connected: boolean}>> {
    return this.post(`/api/agents/${id}/test`, {});
  }
}

export const apiClient = new ApiClient();

// Dedicated client for the Data service (repositories, data-sources)
// Uses NEXT_PUBLIC_DATA_SERVICE_URL or defaults to local dev port 3013
export const dataApiClient = new ApiClient(
  process.env.NEXT_PUBLIC_DATA_SERVICE_URL || 'http://localhost:3013'
);

// Dedicated client for the Billing service
// Uses NEXT_PUBLIC_BILLING_SERVICE_URL, falls back to BILLING_SERVICE_URL, then local dev port 3011
export const billingApiClient = new ApiClient(
  process.env.NEXT_PUBLIC_BILLING_SERVICE_URL ||
  process.env.BILLING_SERVICE_URL ||
  'http://localhost:3011'
);

// Convenience helper: fetch current user via GraphQL when auth is enabled
export async function fetchCurrentUserViaGraphQL() {
  const query = `
    query Me { me { user_id roles } }
  `;
  try {
    const data = await apiClient.graphql<{ me: { user_id: string | null; roles: string[] } }>(query);
    return data.me;
  } catch (err) {
    // In environments where auth is disabled, backend may return default claims.
    // Gracefully return undefined to keep UI stable.
    console.warn('GraphQL me query failed:', err);
    return undefined;
  }
}

/**
 * Safely extract the `data` field from an API response or return the value itself.
 * Avoids use of `any` in callers by operating on unknown.
 */
export function unwrapResponse<T = unknown>(resp: unknown): T | undefined {
  if (typeof resp === 'object' && resp !== null) {
    const r = resp as Record<string, unknown>
    if ('data' in r) return r.data as T
  }
  return resp as T | undefined
}
