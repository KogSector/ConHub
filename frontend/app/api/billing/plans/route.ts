export const dynamic = 'force-dynamic'

import { NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

export async function GET() {
  try {
    const resp = await billingApiClient.get('/api/billing/plans')
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching subscription plans:', error)
    return NextResponse.json({ error: 'Failed to fetch subscription plans' }, { status: 500 })
  }
}