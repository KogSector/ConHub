import { NextRequest, NextResponse } from 'next/server';

export async function POST(req: NextRequest) {
  try {
    const { prompt, context } = await req.json();

    
    await fetch('http://localhost:3001/api/agents/create/cline', {
      method: 'POST',
    });

    const agentsResponse = await fetch('http://localhost:3001/api/agents/list');
    const agents = await agentsResponse.json();
    const clineAgent = agents.find((agent: any) => agent.agent_type === 'cline');

    if (!clineAgent) {
      return NextResponse.json({ error: 'Cline agent not found' }, { status: 404 });
    }

    const response = await fetch(`http://localhost:3001/api/agents/query/${clineAgent.id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, context }),
    });

    const data = await response.json();
    return NextResponse.json({ response: data });
  } catch (error) {
    console.error('Error querying Cline agent:', error);
    return NextResponse.json({ error: 'Failed to query Cline agent' }, { status: 500 });
  }
}
