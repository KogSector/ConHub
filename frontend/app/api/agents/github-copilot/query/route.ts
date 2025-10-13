import { NextRequest, NextResponse } from 'next/server';

export async function POST(req: NextRequest) {
  try {
    const { prompt, context } = await req.json();

    
    await fetch('http://localhost:3001/api/agents/create/github_copilot', {
      method: 'POST',
    });

    const agentsResponse = await fetch('http://localhost:3001/api/agents/list');
    const agents = await agentsResponse.json();
    const copilotAgent = agents.find((agent: any) => agent.agent_type === 'github_copilot');

    if (!copilotAgent) {
      return NextResponse.json({ error: 'GitHub Copilot agent not found' }, { status: 404 });
    }

    const response = await fetch(`http://localhost:3001/api/agents/query/${copilotAgent.id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, context }),
    });

    const data = await response.json();
    return NextResponse.json({ response: data });
  } catch (error) {
    console.error('Error querying GitHub Copilot agent:', error);
    return NextResponse.json({ error: 'Failed to query GitHub Copilot agent' }, { status: 500 });
  }
}