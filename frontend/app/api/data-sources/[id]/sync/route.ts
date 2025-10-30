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

function getData<T = unknown>(resp: unknown): T | undefined {
  if (isApiResp(resp) && 'data' in resp) return resp.data as T
  return undefined
}

export async function POST(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  try {
    const { id } = params

    if (!id) {
      return NextResponse.json(
        { success: false, error: 'Data source ID is required' },
        { status: 400 }
      )
    }

    const resp = await apiClient.post(`/api/data-sources/${id}/sync`, {})
    if (!succeeded(resp)) {
      const err = isApiResp(resp) ? resp.error : undefined
      return NextResponse.json({ success: false, error: err || 'Failed to sync data source' }, { status: 502 })
    }

    return NextResponse.json({ success: true, message: 'Sync started successfully', data: getData(resp) })

  } catch (error) {
    console.error('Error syncing data source:', error)
    return NextResponse.json(
      { success: false, error: 'Internal server error' },
      { status: 500 }
    )
  }
}