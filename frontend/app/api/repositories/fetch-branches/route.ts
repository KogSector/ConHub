import { NextResponse } from 'next/server';
import { list_remote_branches } from '@/lib/git';

export async function POST(request: Request) {
  try {
    const { repoUrl } = await request.json();

    if (!repoUrl || typeof repoUrl !== 'string') {
      return NextResponse.json({ error: 'Repository URL is required.' }, { status: 400 });
    }

    // Basic URL validation
    if (!repoUrl.startsWith('https://') && !repoUrl.startsWith('git@')) {
        return NextResponse.json({ error: 'Invalid repository URL format.' }, { status: 400 });
    }

    const { branches, defaultBranch } = await list_remote_branches(repoUrl);

    return NextResponse.json({ branches, defaultBranch });
  } catch (error: any) {
    console.error('Failed to fetch remote branches:', error);
    return NextResponse.json({ error: error.message || 'An unknown error occurred while fetching branches.' }, { status: 500 });
  }
}
