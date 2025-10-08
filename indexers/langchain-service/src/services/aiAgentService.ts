import { logger } from '../utils/logger';
import { vectorStore } from './vectorStore';

export interface AIAgent {
  id: string;
  name: string;
  type: 'github-copilot' | 'amazon-q' | 'custom';
  status: 'connected' | 'disconnected' | 'error';
  config: any;
  credentials?: any;
}

export interface AgentQuery {
  query: string;
  context?: string;
  agentId?: string;
  includeContext?: boolean;
}

export interface AgentResponse {
  response: string;
  sources?: any[];
  agentUsed: string;
  confidence?: number;
}

class AIAgentService {
  private agents = new Map<string, AIAgent>();

  async registerAgent(agent: Omit<AIAgent, 'id'>): Promise<AIAgent> {
    const id = `agent-${Date.now()}`;
    const newAgent: AIAgent = { ...agent, id };
    this.agents.set(id, newAgent);
    logger.info(`Registered AI agent: ${agent.name} (${agent.type})`);
    return newAgent;
  }

  async getAgents(): Promise<AIAgent[]> {
    return Array.from(this.agents.values());
  }

  async queryAgent(query: AgentQuery): Promise<AgentResponse> {
    const agent = query.agentId ? this.agents.get(query.agentId) : this.getDefaultAgent();
    
    if (!agent) {
      throw new Error('No AI agent available');
    }

    let context = '';
    if (query.includeContext !== false) {
      const results = await vectorStore.similaritySearch(query.query, 3);
      context = results.map(doc => doc.pageContent).join('\n\n');
    }

    switch (agent.type) {
      case 'github-copilot':
        return this.queryGitHubCopilot(query, context, agent);
      case 'amazon-q':
        return this.queryAmazonQ(query, context, agent);
      default:
        return this.queryCustomAgent(query, context, agent);
    }
  }

  private async queryGitHubCopilot(query: AgentQuery, context: string, agent: AIAgent): Promise<AgentResponse> {
    // GitHub Copilot integration would require their API
    return {
      response: `GitHub Copilot response for: ${query.query}\n\nContext: ${context}`,
      agentUsed: agent.name,
      confidence: 0.8
    };
  }

  private async queryAmazonQ(query: AgentQuery, context: string, agent: AIAgent): Promise<AgentResponse> {
    // Amazon Q integration would require their API
    return {
      response: `Amazon Q response for: ${query.query}\n\nContext: ${context}`,
      agentUsed: agent.name,
      confidence: 0.85
    };
  }

  private async queryCustomAgent(query: AgentQuery, context: string, agent: AIAgent): Promise<AgentResponse> {
    return {
      response: `Custom agent response for: ${query.query}\n\nContext: ${context}`,
      agentUsed: agent.name,
      confidence: 0.7
    };
  }

  private getDefaultAgent(): AIAgent | undefined {
    return Array.from(this.agents.values()).find(agent => agent.status === 'connected');
  }
}

export const aiAgentService = new AIAgentService();