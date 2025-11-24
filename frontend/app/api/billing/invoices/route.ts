export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined

    const url = new URL(request.url)
    const query = url.searchParams.toString()
    const endpoint = `/api/billing/invoices${query ? `?${query}` : ''}`

    const resp = await billingApiClient.get(endpoint, authHeader ? { Authorization: authHeader } : undefined)

    // Unwrap common response shapes to return a plain array as the UI expects
    let data: unknown = resp as unknown
    if (typeof data === 'object' && data !== null) {
      const obj = data as Record<string, unknown>
      if (Array.isArray(obj['invoices'])) data = obj['invoices']
      else if (Array.isArray(obj['data'])) data = obj['data']
    }

    return NextResponse.json(data)
  } catch (error) {
    console.error('Error fetching invoices:', error)
    return NextResponse.json({ error: 'Failed to fetch invoices' }, { status: 500 })
  }
}