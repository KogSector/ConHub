import { NextRequest, NextResponse } from 'next/server'
import { readRules, writeRules, AgentRule } from '@/lib/agent-rules'

function json(data: unknown, status = 200) {
  return NextResponse.json({ data }, { status })
}

export async function GET() {
  const rules = readRules()
  return json(rules)
}

export async function POST(req: NextRequest) {
  try {
    const body = await req.json() as { name?: string; content?: string; agentIds?: string[]; enabled?: boolean }
    const name = body.name?.trim() || 'Rule'
    const content = body.content?.trim() || ''
    const agentIds = Array.isArray(body.agentIds) ? body.agentIds.filter(Boolean) : []
    const enabled = body.enabled ?? true

    if (!content) {
      return NextResponse.json({ error: 'Rule content is required' }, { status: 400 })
    }

    const rules = readRules()
    const assigned = new Set<string>()
    for (const r of rules) {
      for (const id of r.agentIds) assigned.add(id)
    }
    for (const id of agentIds) {
      if (assigned.has(id)) {
        return NextResponse.json({ error: `Agent ${id} is already assigned to another rule` }, { status: 409 })
      }
    }

    const newRule: AgentRule = {
      id: `${Date.now()}`,
      name,
      content,
      agentIds,
      enabled,
      createdAt: new Date().toISOString()
    }
    writeRules([...rules, newRule])
    return json(newRule, 201)
  } catch (e) {
    return NextResponse.json({ error: 'Failed to create rule' }, { status: 500 })
  }
}