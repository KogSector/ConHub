export const dynamic = 'force-dynamic'

import { NextResponse } from 'next/server'
import { dataApiClient } from '@/lib/api'

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
    // Call the dedicated data service to avoid recursive self-calls to this route
    const resp = await dataApiClient.get('/api/data-sources')
    if (!succeeded(resp)) {
      const err = isApiResp(resp) ? resp.error : undefined
      return NextResponse.json({ success: false, error: err || 'Failed to fetch data sources' }, { status: 502 })
    }

    const data = getData<unknown[]>(resp) || []
    return NextResponse.json({ success: true, data: { dataSources: data } })

  } catch (error) {
    console.error('Error fetching data sources:', error)
    // In development, provide a graceful fallback so the UI can render
    if (process.env.NODE_ENV === 'development') {
      return NextResponse.json({ success: true, data: { dataSources: [] } })
    }
    return NextResponse.json({ success: false, error: 'Internal server error' }, { status: 500 })
  }
}