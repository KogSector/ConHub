export const dynamic = 'force-dynamic'

import { NextResponse } from 'next/server'
import { apiClient } from '@/lib/api'

type ApiResp = { success?: boolean; error?: string; data?: unknown }

function isApiResp(obj: unknown): obj is ApiResp {
  return typeof obj === 'object' && obj !== null
}

function succeeded(resp: unknown): boolean {
  return isApiResp(resp) && resp.success === true
}

export async function GET() {
  try {
    const resp = await apiClient.get('/api/billing/plans')
    if (!succeeded(resp)) throw new Error('Failed to fetch plans')
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching subscription plans:', error)
    return NextResponse.json({ error: 'Failed to fetch subscription plans' }, { status: 500 })
  }
}