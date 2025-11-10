export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    if (!authHeader) return NextResponse.json({ error: 'Authorization required' }, { status: 401 })

    const resp = await billingApiClient.get('/api/billing/subscription', { Authorization: authHeader })
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching subscription:', error)
    return NextResponse.json({ error: 'Failed to fetch subscription' }, { status: 500 })
  }
}

export async function POST(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    if (!authHeader) return NextResponse.json({ error: 'Authorization required' }, { status: 401 })

    const body = await request.json()
    const resp = await billingApiClient.post('/api/billing/subscription', body, { Authorization: authHeader })
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error creating subscription:', error)
    return NextResponse.json({ error: 'Failed to create subscription' }, { status: 500 })
  }
}

export async function PUT(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    if (!authHeader) return NextResponse.json({ error: 'Authorization required' }, { status: 401 })

    const body = await request.json()
    const resp = await billingApiClient.put('/api/billing/subscription', body, { Authorization: authHeader })
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error updating subscription:', error)
    return NextResponse.json({ error: 'Failed to update subscription' }, { status: 500 })
  }
}