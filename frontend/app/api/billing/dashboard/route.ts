export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    if (!authHeader) {
      return NextResponse.json({ error: 'Authorization required' }, { status: 401 })
    }

    const resp = await billingApiClient.get('/api/billing/dashboard', { Authorization: authHeader })
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching billing dashboard:', error)
    return NextResponse.json({ error: 'Failed to fetch billing dashboard' }, { status: 500 })
  }
}