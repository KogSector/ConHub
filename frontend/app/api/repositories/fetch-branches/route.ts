import { NextResponse } from 'next/server';

export async function POST(request: Request) {
  try {
    const { repoUrl } = await request.json();

    if (!repoUrl || typeof repoUrl !== 'string') {
      return NextResponse.json({ error: 'Repository URL is required.' }, { status: 400 });
    }

    // First validate the URL
    const validateResponse = await fetch('http://localhost:3001/api/repositories/validate-url', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ repo_url: repoUrl })
    });

    const validateData = await validateResponse.json();
    
    if (!validateData.success) {
      return NextResponse.json({ 
        error: validateData.error || 'Invalid repository URL' 
      }, { status: 400 });
    }

    // Then fetch branches
    const branchesResponse = await fetch('http://localhost:3001/api/repositories/fetch-branches', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        repo_url: repoUrl,
        credentials: null // For public repos
      })
    });

    const branchesData = await branchesResponse.json();
    
    if (!branchesData.success) {
      return NextResponse.json({ 
        error: branchesData.error || 'Failed to fetch branches' 
      }, { status: branchesResponse.status });
    }

    return NextResponse.json({ 
      branches: branchesData.data.branches,
      defaultBranch: branchesData.data.default_branch,
      provider: validateData.data.provider,
      repoInfo: validateData.data
    });
  } catch (error: any) {
    console.error('Failed to fetch remote branches:', error);
    return NextResponse.json({ 
      error: error.message || 'An unknown error occurred while fetching branches.' 
    }, { status: 500 });
  }
}
