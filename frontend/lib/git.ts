import git from 'isomorphic-git';
import http from 'isomorphic-git/http/node';

// A simple in-memory cache to avoid re-fetching for the same URL within a short time
const cache = new Map<string, { branches: string[], defaultBranch: string, timestamp: number }>();
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

export async function list_remote_branches(repoUrl: string): Promise<{ branches: string[], defaultBranch: string }> {
  const cached = cache.get(repoUrl);
  if (cached && (Date.now() - cached.timestamp < CACHE_TTL)) {
    return { branches: cached.branches, defaultBranch: cached.defaultBranch };
  }

  try {
    const remoteInfo = await git.getRemoteInfo({
      http,
      url: repoUrl,
    });

    if (!remoteInfo.refs.heads) {
      throw new Error("No branches found in the remote repository.");
    }

    const branches = Object.keys(remoteInfo.refs.heads);
    const defaultBranch = remoteInfo.HEAD ? remoteInfo.HEAD.replace('refs/heads/', '') : branches[0];

    // Update cache
    cache.set(repoUrl, { branches, defaultBranch, timestamp: Date.now() });

    return { branches, defaultBranch };
  } catch (error: any) {
    console.error(`Error fetching branches for ${repoUrl}:`, error);
    // Provide more specific error messages
    if (error.message.includes('404') || error.message.includes('not found')) {
        throw new Error("Repository not found. Please check the URL.");
    }
    if (error.message.includes('authentication required')) {
        throw new Error("This repository is private and requires authentication, which is not yet supported for branch fetching.");
    }
    throw new Error("Could not connect to the repository. Please check the URL and your network connection.");
  }
}
