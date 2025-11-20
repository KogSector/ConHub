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

    await apiClient.post('/api/agents/create/github_copilot', {})

    const agentsResp = await apiClient.get('/api/agents')
    const rawAgents = (typeof agentsResp === 'object' && agentsResp !== null && 'data' in agentsResp) ? (agentsResp as Record<string, unknown>)['data'] : agentsResp
    const agents = extractAgents(rawAgents)
    const copilotAgent = agents.find(agent => agent.agent_type === 'github_copilot')

    if (!copilotAgent) {
      return NextResponse.json({ error: 'GitHub Copilot agent not found' }, { status: 404 })
    }

    const rules = getRulesForAgent(copilotAgent.id)
    const resp = await apiClient.post(`/api/agents/query/${copilotAgent.id}`, { prompt, context, rules })
    const data = (typeof resp === 'object' && resp !== null && 'data' in resp) ? (resp as Record<string, unknown>)['data'] : resp
    return NextResponse.json({ response: data })
  } catch (error) {
    console.error('Error querying GitHub Copilot agent:', error)
    return NextResponse.json({ error: 'Failed to query GitHub Copilot agent' }, { status: 500 })
  }
}