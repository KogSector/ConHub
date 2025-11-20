import { NextRequest, NextResponse } from 'next/server'
import { removeConnection, readConnections } from '@/lib/connections-store'

export async function DELETE(_: NextRequest, { params }: { params: { id: string } }) {
  removeConnection(params.id)
  return NextResponse.json({ success: true })
}

export async function GET(_: NextRequest, { params }: { params: { id: string } }) {
  const conn = readConnections().find(c => c.id === params.id)
  return NextResponse.json({ data: conn || null })
}