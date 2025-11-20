import fs from 'fs'
import path from 'path'

export type AgentRule = {
  id: string
  name: string
  content: string
  agentIds: string[]
  enabled: boolean
  createdAt: string
}

const dataFile = path.join(process.cwd(), 'frontend', 'data', 'agent-rules.json')

export function readRules(): AgentRule[] {
  try {
    if (!fs.existsSync(dataFile)) return []
    const raw = fs.readFileSync(dataFile, 'utf8')
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? parsed as AgentRule[] : []
  } catch {
    return []
  }
}

export function writeRules(rules: AgentRule[]): void {
  try {
    const dir = path.dirname(dataFile)
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true })
    fs.writeFileSync(dataFile, JSON.stringify(rules, null, 2), 'utf8')
  } catch {}
}

export function getRulesForAgent(agentId: string): AgentRule[] {
  return readRules().filter(r => r.enabled && r.agentIds.includes(agentId))
}