import { NextRequest, NextResponse } from 'next/server';

const DATA_SERVICE_URL = process.env.NEXT_PUBLIC_DATA_SERVICE_URL || process.env.DATA_SERVICE_URL || 'http://localhost:3013';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    
    // Forward authorization header
    const authHeader = request.headers.get('authorization');
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (authHeader) {
      headers['Authorization'] = authHeader;
    }

    const response = await fetch(`${DATA_SERVICE_URL}/api/github/sync`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    const data = await response.json();

    if (!response.ok) {
      return NextResponse.json(data, { status: response.status });
    }

    return NextResponse.json(data);
  } catch (error) {
    console.error('GitHub sync proxy error:', error);
    return NextResponse.json(
      { error: 'Failed to sync repository', details: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}
