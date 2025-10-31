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

function getData<T = unknown>(resp: unknown): T | undefined {
  if (isApiResp(resp) && 'data' in resp) return resp.data as T
  return undefined
}

export async function GET() {
  try {
    const resp = await apiClient.get('/api/data-sources')
    if (!succeeded(resp)) {
      const err = isApiResp(resp) ? resp.error : undefined
      return NextResponse.json({ success: false, error: err || 'Failed to fetch data sources' }, { status: 502 })
    }

    const data = getData<unknown[]>(resp) || []
    return NextResponse.json({ success: true, dataSources: data })

  } catch (error) {
    console.error('Error fetching data sources:', error)
    return NextResponse.json(
      { success: false, error: 'Internal server error' },
      { status: 500 }
    )
  }
}