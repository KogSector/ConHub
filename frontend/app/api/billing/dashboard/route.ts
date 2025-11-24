export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined
    const resp = await billingApiClient.get('/api/billing/dashboard', authHeader ? { Authorization: authHeader } : undefined)
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching billing dashboard:', error)
    return NextResponse.json({ error: 'Failed to fetch billing dashboard' }, { status: 500 })
  }
}