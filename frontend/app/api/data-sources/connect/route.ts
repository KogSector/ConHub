import { NextRequest, NextResponse } from 'next/server';


function extractRepositoryName(url: string): string | null {
  try {
    const urlObj = new URL(url);
    const pathParts = urlObj.pathname.split('/').filter(Boolean);
    
    
    
    if (pathParts.length >= 2) {
      const owner = pathParts[pathParts.length - 2];
      const repo = pathParts[pathParts.length - 1].replace('.git', '');
      return `${owner}/${repo}`;
    }
    return null;
  } catch {
    return null;
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { type, url, credentials, config } = body;

    console.log('API Route - Received request:', { type, url, credentials: credentials ? 'present' : 'missing', config });

    
    if (!type) {
      console.log('API Route - Missing type');
      return NextResponse.json(
        { success: false, error: 'Data source type is required' },
        { status: 400 }
      );
    }

    if (!url && (type === 'github' || type === 'bitbucket')) {
      console.log('API Route - Missing URL for VCS');
      return NextResponse.json(
        { success: false, error: 'Repository URL is required' },
        { status: 400 }
      );
    }

    
    if (type === 'github' && !credentials?.accessToken) {
      console.log('API Route - Missing GitHub token');
      return NextResponse.json(
        { success: false, error: 'GitHub access token is required' },
        { status: 400 }
      );
    }

    if (type === 'bitbucket' && (!credentials?.username || !credentials?.appPassword)) {
      console.log('API Route - Missing Bitbucket credentials');
      return NextResponse.json(
        { success: false, error: 'BitBucket username and app password are required' },
        { status: 400 }
      );
    }

    
    let processedConfig = config;
    if ((type === 'github' || type === 'bitbucket') && url) {
      const repoName = extractRepositoryName(url);
      if (repoName) {
        processedConfig = {
          ...config,
          repositories: [repoName], 
          url 
        };
        console.log('API Route - Extracted repository:', repoName);
      } else {
        console.log('API Route - Failed to extract repository from URL:', url);
        return NextResponse.json(
          { success: false, error: 'Invalid repository URL format' },
          { status: 400 }
        );
      }
    }

    
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:3001';
    console.log('API Route - Forwarding to backend:', backendUrl);
    
    const payload = {
      source_type: type,
      url,
      credentials,
      config: processedConfig
    };
    
    console.log('API Route - Sending payload to backend:', payload);
    
    const response = await fetch(`${backendUrl}/api/data-sources/connect`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    console.log('API Route - Backend response status:', response.status);
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.log('API Route - Backend error:', errorData);
      return NextResponse.json(
        { success: false, error: errorData.message || 'Failed to connect data source' },
        { status: response.status }
      );
    }

    const result = await response.json();
    console.log('API Route - Backend success:', result);
    
    return NextResponse.json({
      success: true,
      message: 'Repository connected successfully',
      data: result.data
    });

  } catch (error) {
    console.error('Error connecting data source:', error);
    return NextResponse.json(
      { success: false, error: 'Internal server error' },
      { status: 500 }
    );
  }
}