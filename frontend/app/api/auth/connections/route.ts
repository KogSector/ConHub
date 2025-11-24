import { NextRequest, NextResponse } from 'next/server'
import { apiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization') || undefined
    const resp = await apiClient.get('/api/auth/connections', authHeader ? { Authorization: authHeader } : {})
    return NextResponse.json(resp)
  } catch (error) {
    return NextResponse.json({ success: true, data: [] })
  }
}