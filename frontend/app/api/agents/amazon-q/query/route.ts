import { NextRequest, NextResponse } from 'next/server';
import { apiClient } from '@/lib/api';
import { getRulesForAgent } from '@/lib/agent-rules'

type Agent = { id: string; agent_type: string }

function isAgent(obj: unknown): obj is Agent {
  return typeof obj === 'object' && obj !== null && typeof (obj as Record<string, unknown>)['id'] === 'string' && typeof (obj as Record<string, unknown>)['agent_type'] === 'string'
}

function extractAgents(value: unknown): Agent[] {
  if (Array.isArray(value)) {
    return value.filter(isAgent)
  }
  return []
}

export async function POST(req: NextRequest) {
  try {
    const body = await req.json() as { prompt?: string; context?: unknown }
    const prompt = body.prompt ?? ''
    const context = body.context

    await apiClient.post('/api/agents/create/amazon_q', {})

    const agentsResp = await apiClient.get('/api/agents')
    const rawAgents = (typeof agentsResp === 'object' && agentsResp !== null && 'data' in agentsResp) ? (agentsResp as Record<string, unknown>)['data'] : agentsResp
    const agents = extractAgents(rawAgents)
    const amazonQAgent = agents.find(agent => agent.agent_type === 'amazon_q')

    if (!amazonQAgent) {
      return NextResponse.json({ error: 'Amazon Q agent not found' }, { status: 404 })
    }

    const rules = getRulesForAgent(amazonQAgent.id)
    const resp = await apiClient.post(`/api/agents/query/${amazonQAgent.id}`, { prompt, context, rules })
    const data = (typeof resp === 'object' && resp !== null && 'data' in resp) ? (resp as Record<string, unknown>)['data'] : resp
    return NextResponse.json({ response: data })
  } catch (error) {
    console.error('Error querying Amazon Q agent:', error)
    return NextResponse.json({ error: 'Failed to query Amazon Q agent' }, { status: 500 })
  }
}