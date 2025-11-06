export const dynamic = 'force-dynamic'

import { NextResponse } from 'next/server'
import { dataApiClient } from '@/lib/api'

type ApiResp = { success?: boolean; error?: string; data?: unknown }

function isApiResp(obj: unknown): obj is ApiResp {
  return typeof obj === 'object' && obj !== null
}

function getData<T = unknown>(resp: unknown): T | undefined {
  if (isApiResp(resp) && 'data' in resp) {
    return resp.data as T
  }
  return undefined
}

function succeeded(resp: unknown): boolean {
  if (isApiResp(resp) && typeof resp.success === 'boolean') return resp.success
  return false
}

export async function POST(request: Request) {
  try {
    const body = await request.json() as { repoUrl?: unknown }
    const repoUrl = typeof body.repoUrl === 'string' ? body.repoUrl : ''

    if (!repoUrl) {
      return NextResponse.json({ error: 'Repository URL is required.' }, { status: 400 })
    }

    const branchesData = await dataApiClient.post('/api/data/sources/branches', { repoUrl: repoUrl, credentials: null })
    if (!succeeded(branchesData)) {
      const err = isApiResp(branchesData) ? branchesData.error : undefined
      return NextResponse.json({ error: err || 'Failed to fetch branches' }, { status: 502 })
    }

    const branches = getData<{ branches?: unknown[]; default_branch?: string }>(branchesData)

    return NextResponse.json({
      branches: branches?.branches,
      defaultBranch: branches?.default_branch,
      provider: undefined,
      repoInfo: undefined,
    })
  } catch (error) {
    console.error('Failed to fetch remote branches:', error)
    const message = (error && typeof error === 'object' && typeof (error as Record<string, unknown>)['message'] === 'string') ? (error as Record<string, unknown>)['message'] as string : 'An unknown error occurred while fetching branches.'
    return NextResponse.json({ error: message }, { status: 500 })
  }
}
