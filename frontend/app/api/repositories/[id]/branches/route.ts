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

export async function GET(
  request: Request,
  { params }: { params: { id: string } }
) {
  try {
    const repositoryId = params.id

    if (!repositoryId) {
      return NextResponse.json({ error: 'Repository ID is required.' }, { status: 400 })
    }

    // TODO: Implement actual API call to fetch branches for the specific connected repository
    // This should call the data service to get branches for the repository by ID
    const branchesData = await dataApiClient.get(`/api/data/sources/${repositoryId}/branches`)
    
    if (!succeeded(branchesData)) {
      const err = isApiResp(branchesData) ? branchesData.error : undefined
      return NextResponse.json({ error: err || 'Failed to fetch branches' }, { status: 502 })
    }

    const branches = getData<{ branches?: string[]; default_branch?: string }>(branchesData)

    return NextResponse.json({
      branches: branches?.branches || ['main', 'master', 'develop'],
      defaultBranch: branches?.default_branch || 'main',
    })
  } catch (error) {
    console.error('Failed to fetch repository branches:', error)
    const message = (error && typeof error === 'object' && typeof (error as Record<string, unknown>)['message'] === 'string') 
      ? (error as Record<string, unknown>)['message'] as string 
      : 'An unknown error occurred while fetching branches.'
    return NextResponse.json({ error: message }, { status: 500 })
  }
}