"use client"

import { useEffect, useMemo, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { apiClient, unwrapResponse } from '@/lib/api'
import Link from 'next/link'

type Agent = { id: string; agent_type: string; name?: string }
type Rule = { id: string; name: string; content: string; agentIds: string[]; enabled: boolean; createdAt: string }

export default function AgentRulesPage() {
  const [agents, setAgents] = useState<Agent[]>([])
  const [rules, setRules] = useState<Rule[]>([])
  const [name, setName] = useState('')
  const [content, setContent] = useState('')
  const [selectedAgents, setSelectedAgents] = useState<string[]>([])
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')

  useEffect(() => {
    const load = async () => {
      try {
        const agentsResp = await apiClient.get('/api/agents')
        const raw = (typeof agentsResp === 'object' && agentsResp !== null && 'data' in agentsResp) ? (agentsResp as any).data : agentsResp
        setAgents(Array.isArray(raw) ? raw as Agent[] : [])
      } catch {}
      try {
        const r = await apiClient.get('/api/agent-rules')
        const list = unwrapResponse<Rule[]>(r) ?? []
        setRules(list)
      } catch {}
    }
    load()
  }, [])

  const assignedAgentIds = useMemo(() => new Set(rules.flatMap(r => r.agentIds)), [rules])

  const addRule = async () => {
    setError('')
    if (!content.trim()) { setError('Rule content is required'); return }
    setSaving(true)
    try {
      const resp = await apiClient.post('/api/agent-rules', {
        name: name.trim() || 'Agent Rule',
        content: content.trim(),
        agentIds: selectedAgents,
        enabled: true
      })
      const created = unwrapResponse<Rule>(resp)
      if (created) {
        setRules(prev => [...prev, created])
        setName('')
        setContent('')
        setSelectedAgents([])
      }
    } catch (e: any) {
      setError(e?.message || 'Failed to create rule')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-8 space-y-6">
      <div className="flex items-center gap-2">
        <Link href="/dashboard">
          <Button variant="ghost" size="sm">Dashboard</Button>
        </Link>
        <div className="h-6 w-px bg-border" />
        <h1 className="text-2xl font-bold">Agent Rules</h1>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Create Rule</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <div className="p-3 bg-red-50 border border-red-200 rounded-md text-sm text-red-700">{error}</div>
          )}
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="ruleName">Rule Name</Label>
              <Input id="ruleName" value={name} onChange={e => setName(e.target.value)} placeholder="Eg. Safe coding guidelines" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="agentSelect">Select Agent</Label>
              <Select onValueChange={(val) => {
                if (!assignedAgentIds.has(val) && !selectedAgents.includes(val)) setSelectedAgents([val])
              }}>
                <SelectTrigger>
                  <SelectValue placeholder="Choose one connected agent" />
                </SelectTrigger>
                <SelectContent>
                  {agents.map(a => (
                    <SelectItem key={a.id} value={a.id} disabled={assignedAgentIds.has(a.id)}>
                      {(a.name || a.agent_type)} {assignedAgentIds.has(a.id) ? '• already assigned' : ''}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">An agent cannot be assigned to more than one rule.</p>
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="content">Rule Content</Label>
            <Textarea id="content" value={content} onChange={e => setContent(e.target.value)} placeholder="Write the rule the agents must follow..." className="min-h-[120px]" />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => { setName(''); setContent(''); setSelectedAgents([]) }}>Cancel</Button>
            <Button onClick={addRule} disabled={saving}>{saving ? 'Saving...' : 'Save Rule'}</Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Configured Rules ({rules.length})</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {rules.length === 0 && (
            <div className="text-sm text-muted-foreground">No rules added yet.</div>
          )}
          {rules.map(r => (
            <div key={r.id} className="p-3 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between">
                <div className="font-medium">{r.name}</div>
                <div className="text-xs text-muted-foreground">{new Date(r.createdAt).toLocaleString()}</div>
              </div>
              <div className="text-sm text-muted-foreground mt-1">Agents: {r.agentIds.length > 0 ? r.agentIds.join(', ') : 'None'}</div>
              <div className="text-sm mt-2 whitespace-pre-wrap">{r.content}</div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  )
}