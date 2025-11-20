import { NextRequest, NextResponse } from 'next/server'
import { upsertConnection } from '@/lib/connections-store'

export async function POST(req: NextRequest) {
  try {
    const { provider } = await req.json() as { provider?: string; code?: string }
    if (!provider) return NextResponse.json({ error: 'Missing provider' }, { status: 400 })
    const conn = upsertConnection(provider as any, 'demo')
    return NextResponse.json({ data: conn })
  } catch {
    return NextResponse.json({ error: 'Exchange failed' }, { status: 500 })
  }
}