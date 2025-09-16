import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { type, credentials, config } = body;

    // Validate required fields
    if (!type) {
      return NextResponse.json(
        { success: false, error: 'Data source type is required' },
        { status: 400 }
      );
    }

    // Validate credentials based on type
    if (type === 'github' && !credentials?.accessToken) {
      return NextResponse.json(
        { success: false, error: 'GitHub access token is required' },
        { status: 400 }
      );
    }

    if (type === 'bitbucket' && (!credentials?.username || !credentials?.appPassword)) {
      return NextResponse.json(
        { success: false, error: 'BitBucket username and app password are required' },
        { status: 400 }
      );
    }

    // Forward to backend service
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:3001';
    const response = await fetch(`${backendUrl}/api/data-sources/connect`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        source_type: type,
        credentials,
        config
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        { success: false, error: errorData.message || 'Failed to connect data source' },
        { status: response.status }
      );
    }

    const result = await response.json();
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