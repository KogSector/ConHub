export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { billingApiClient } from '@/lib/api'

const DEMO_USER_ID = '550e8400-e29b-41d4-a716-446655440000'

export async function GET(request: NextRequest) {
  try {
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined
    const url = new URL(request.url)
    const customerId = url.searchParams.get('customer_id') || DEMO_USER_ID

    const endpoint = `/api/billing/customers/${customerId}/payment-methods`
    const resp = await billingApiClient.get(endpoint, authHeader ? { Authorization: authHeader } : undefined)

    let data: unknown = resp as unknown
    if (typeof data === 'object' && data !== null) {
      const obj = data as Record<string, unknown>
      if (Array.isArray(obj['payment_methods'])) data = obj['payment_methods']
      else if (Array.isArray(obj['data'])) data = obj['data']
    }

    return NextResponse.json(data)
  } catch (error) {
    console.error('Error fetching payment methods:', error)
    return NextResponse.json({ error: 'Failed to fetch payment methods' }, { status: 500 })
  }
}

export async function POST(request: NextRequest) {
  try {
    let authHeader = request.headers.get('authorization') || undefined
    if (authHeader && /null|undefined/i.test(authHeader)) authHeader = undefined
    const body = await request.json()
    if (typeof body?.type === 'string') {
      const t = body.type.toLowerCase()
      body.type = t === 'card' ? 'Card' : t === 'paypal' ? 'Paypal' : t === 'bank_account' ? 'BankAccount' : body.type
    }
    const resp = await billingApiClient.post('/api/billing/payment-methods', body, authHeader ? { Authorization: authHeader } : undefined)
    return NextResponse.json(resp, { status: 201 })
  } catch (error) {
    console.error('Error adding payment method:', error)
    return NextResponse.json({ error: 'Failed to add payment method' }, { status: 500 })
  }
}