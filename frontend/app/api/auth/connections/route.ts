import { NextResponse } from 'next/server'
import { readConnections } from '@/lib/connections-store'

export async function GET() {
  const data = readConnections()
  return NextResponse.json({ data })
}