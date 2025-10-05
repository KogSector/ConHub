import { NextRequest, NextResponse } from 'next/server';

export async function POST(req: NextRequest) {
  try {
    const { prompt, context } = await req.json();

    // Create the agent if it doesn't exist
    await fetch('http://localhost:3001/api/agents/create/amazon_q', {
      method: 'POST',
    });

    const agentsResponse = await fetch('http://localhost:3001/api/agents/list');
    const agents = await agentsResponse.json();
    const amazonQAgent = agents.find((agent: any) => agent.agent_type === 'amazon_q');

    if (!amazonQAgent) {
      return NextResponse.json({ error: 'Amazon Q agent not found' }, { status: 404 });
    }

    const response = await fetch(`http://localhost:3001/api/agents/query/${amazonQAgent.id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, context }),
    });

    const data = await response.json();
    return NextResponse.json({ response: data });
  } catch (error) {
    console.error('Error querying Amazon Q agent:', error);
    return NextResponse.json({ error: 'Failed to query Amazon Q agent' }, { status: 500 });
  }
}