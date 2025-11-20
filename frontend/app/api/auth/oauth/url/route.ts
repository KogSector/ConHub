import { NextRequest, NextResponse } from 'next/server'

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url)
  const provider = searchParams.get('provider') || 'github'
  const origin = process.env.NEXT_PUBLIC_BASE_URL || 'http://localhost:3000'
  const url = `${origin}/auth/callback?provider=${encodeURIComponent(provider)}&code=dummy`
  return NextResponse.json({ url, state: `${provider}-${Date.now()}` })
}