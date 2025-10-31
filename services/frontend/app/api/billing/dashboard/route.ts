export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { apiClient } from '@/lib/api'

type ApiResp = { success?: boolean; error?: string; data?: unknown }

function isApiResp(obj: unknown): obj is ApiResp {
  return typeof obj === 'object' && obj !== null
}

function succeeded(resp: unknown): boolean {
  return isApiResp(resp) && resp.success === true
}

export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    if (!authHeader) {
      return NextResponse.json({ error: 'Authorization required' }, { status: 401 })
    }

    const resp = await apiClient.get('/api/billing/dashboard', { Authorization: authHeader })
    if (!succeeded(resp)) {
      const err = isApiResp(resp) ? resp.error : undefined
      return NextResponse.json({ error: err || 'Failed to fetch billing dashboard' }, { status: 502 })
    }

    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching billing dashboard:', error)
    return NextResponse.json({ error: 'Failed to fetch billing dashboard' }, { status: 500 })
  }
}