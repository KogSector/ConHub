export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined
    const resp = await billingApiClient.get('/api/billing/subscription', authHeader ? { Authorization: authHeader } : undefined)
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error fetching subscription:', error)
    return NextResponse.json({ error: 'Failed to fetch subscription' }, { status: 500 })
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined
    const resp = await billingApiClient.post('/api/billing/subscription', body, authHeader ? { Authorization: authHeader } : undefined)
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error creating subscription:', error)
    return NextResponse.json({ error: 'Failed to create subscription' }, { status: 500 })
  }
}

export async function PUT(request: NextRequest) {
  try {
    const body = await request.json()
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined
    const resp = await billingApiClient.put('/api/billing/subscription', body, authHeader ? { Authorization: authHeader } : undefined)
    return NextResponse.json(resp)
  } catch (error) {
    console.error('Error updating subscription:', error)
    return NextResponse.json({ error: 'Failed to update subscription' }, { status: 500 })
  }
}