import { NextRequest, NextResponse } from 'next/server';

export async function POST(req: NextRequest) {
  try {
    const { prompt, context } = await req.json();

    // Create the agent if it doesn't exist
    await fetch('http://localhost:3001/api/agents/create/cursor_ide', {
      method: 'POST',
    });

    const agentsResponse = await fetch('http://localhost:3001/api/agents/list');
    const agents = await agentsResponse.json();
    const cursorAgent = agents.find((agent: any) => agent.agent_type === 'cursor_ide');

    if (!cursorAgent) {
      return NextResponse.json({ error: 'Cursor IDE agent not found' }, { status: 404 });
    }

    const response = await fetch(`http://localhost:3001/api/agents/query/${cursorAgent.id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, context }),
    });

    const data = await response.json();
    return NextResponse.json({ response: data });
  } catch (error) {
    console.error('Error querying Cursor IDE agent:', error);
    return NextResponse.json({ error: 'Failed to query Cursor IDE agent' }, { status: 500 });
  }
}