import { NextRequest, NextResponse } from 'next/server'
import { apiClient } from '@/lib/api'


function extractRepositoryName(url: string): string | null {
  try {
    const urlObj = new URL(url)
    const pathParts = urlObj.pathname.split('/').filter(Boolean)

    if (pathParts.length >= 2) {
      const owner = pathParts[pathParts.length - 2]
      const repo = pathParts[pathParts.length - 1].replace('.git', '')
      return `${owner}/${repo}`
    }
    return null
  } catch {
    return null
  }
}

type ApiResp = { success?: boolean; error?: string; data?: unknown }

function isApiResp(obj: unknown): obj is ApiResp {
  return typeof obj === 'object' && obj !== null
}

function succeeded(resp: unknown): boolean {
  return isApiResp(resp) && resp.success === true
}

function getData<T = unknown>(resp: unknown): T | undefined {
  if (isApiResp(resp) && 'data' in resp) return resp.data as T
  return undefined
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json() as Record<string, unknown>
    const type = typeof body.type === 'string' ? body.type : undefined
    const url = typeof body.url === 'string' ? body.url : undefined
    const credentials = body.credentials as Record<string, unknown> | undefined
    const config = body.config as Record<string, unknown> | undefined

    if (!type) {
      return NextResponse.json(
        { success: false, error: 'Data source type is required' },
        { status: 400 }
      )
    }

    if (!url && (type === 'github' || type === 'bitbucket')) {
      return NextResponse.json(
        { success: false, error: 'Repository URL is required' },
        { status: 400 }
      )
    }

    if (type === 'github' && !(credentials && typeof credentials['accessToken'] === 'string')) {
      return NextResponse.json(
        { success: false, error: 'GitHub access token is required' },
        { status: 400 }
      )
    }

    if (type === 'bitbucket' && !(credentials && typeof credentials['username'] === 'string' && typeof credentials['appPassword'] === 'string')) {
      return NextResponse.json(
        { success: false, error: 'BitBucket username and app password are required' },
        { status: 400 }
      )
    }

    let processedConfig = config
    if ((type === 'github' || type === 'bitbucket') && url) {
      const repoName = extractRepositoryName(url)
      if (repoName) {
        processedConfig = {
          ...config,
          repositories: [repoName],
          url,
        }
      } else {
        return NextResponse.json(
          { success: false, error: 'Invalid repository URL format' },
          { status: 400 }
        )
      }
    }

    const payload = {
      source_type: type,
      url,
      credentials,
      config: processedConfig,
    }

    const resp = await apiClient.post('/api/data-sources/connect', payload)
    if (!succeeded(resp)) {
      const err = isApiResp(resp) ? resp.error : undefined
      return NextResponse.json({ success: false, error: err || 'Failed to connect data source' }, { status: 502 })
    }

    return NextResponse.json({ success: true, message: 'Repository connected successfully', data: getData(resp) })

  } catch (error) {
    console.error('Error connecting data source:', error)
    return NextResponse.json(
      { success: false, error: 'Internal server error' },
      { status: 500 }
    )
  }
}