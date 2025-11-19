export const dynamic = 'force-dynamic'

import { NextRequest, NextResponse } from 'next/server'
import { dataApiClient } from '@/lib/api'

export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    if (!authHeader) {
      return NextResponse.json({ error: 'Authorization required' }, { status: 401 })
    }

    // Fetch connected accounts from data service
    const resp = await dataApiClient.get('/api/data/accounts', { Authorization: authHeader })
    
    if (!resp.success) {
      return NextResponse.json({ 
        success: true, 
        data: { repositories: [] } 
      })
    }

    // Transform connected accounts to repository format
    const repositories = (resp.data?.accounts || [])
      .filter((account: any) => ['github', 'gitlab', 'bitbucket'].includes(account.connector_type))
      .map((account: any) => {
        const metadata = account.metadata || {}
        const repoInfo = metadata.repo_info || {}
        
        return {
          id: account.id,
          name: repoInfo.name || account.account_name,
          description: repoInfo.description || `${account.connector_type} repository`,
          language: (repoInfo.languages && repoInfo.languages[0]) || 'Unknown',
          stars: repoInfo.stars || 0,
          forks: repoInfo.forks || 0,
          lastUpdated: account.last_sync_at ? new Date(account.last_sync_at).toLocaleDateString() : 'Never',
          status: account.status === 'connected' ? 'active' : 
                 account.status === 'syncing' ? 'syncing' : 
                 account.status === 'error' ? 'error' : 'inactive',
          url: repoInfo.html_url || `https://${account.connector_type}.com/${account.account_identifier}`,
          provider: account.connector_type
        }
      })

    return NextResponse.json({
      success: true,
      data: { repositories }
    })
  } catch (error) {
    console.error('Error fetching repositories:', error)
    return NextResponse.json({ 
      success: true, 
      data: { repositories: [] } 
    })
  }
}
