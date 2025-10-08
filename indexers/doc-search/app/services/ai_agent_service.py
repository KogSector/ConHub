from typing import Dict, List, Optional, Any
from pydantic import BaseModel
from enum import Enum
import logging
from datetime import datetime

logger = logging.getLogger(__name__)

class AgentType(str, Enum):
    GITHUB_COPILOT = "github-copilot"
    AMAZON_Q = "amazon-q"
    OPENAI_GPT = "openai-gpt"
    ANTHROPIC_CLAUDE = "anthropic-claude"
    CUSTOM = "custom"

class AgentStatus(str, Enum):
    CONNECTED = "connected"
    DISCONNECTED = "disconnected"
    ERROR = "error"

class AIAgent(BaseModel):
    id: str
    name: str
    type: AgentType
    status: AgentStatus
    config: Dict[str, Any]
    credentials: Optional[Dict[str, str]] = None
    created_at: datetime
    updated_at: datetime

class AgentQuery(BaseModel):
    query: str
    context: Optional[str] = None
    agent_id: Optional[str] = None
    include_context: bool = True
    max_tokens: Optional[int] = 1000
    temperature: Optional[float] = 0.7

class AgentResponse(BaseModel):
    response: str
    sources: Optional[List[Dict[str, Any]]] = None
    agent_used: str
    confidence: Optional[float] = None
    tokens_used: Optional[int] = None
    processing_time: Optional[float] = None

class AIAgentService:
    def __init__(self):
        self.agents: Dict[str, AIAgent] = {}
        self._initialize_default_agents()

    def _initialize_default_agents(self):
        """Initialize default AI agents"""
        default_agents = [
            {
                "name": "GitHub Copilot",
                "type": AgentType.GITHUB_COPILOT,
                "status": AgentStatus.DISCONNECTED,
                "config": {
                    "model": "copilot-chat",
                    "max_tokens": 1000,
                    "temperature": 0.7
                }
            },
            {
                "name": "Amazon Q Developer",
                "type": AgentType.AMAZON_Q,
                "status": AgentStatus.DISCONNECTED,
                "config": {
                    "model": "amazon-q",
                    "max_tokens": 1000,
                    "temperature": 0.7
                }
            }
        ]

        for agent_data in default_agents:
            agent_id = f"agent-{agent_data['type']}-{int(datetime.now().timestamp())}"
            agent = AIAgent(
                id=agent_id,
                created_at=datetime.now(),
                updated_at=datetime.now(),
                **agent_data
            )
            self.agents[agent_id] = agent

    async def register_agent(self, agent_data: Dict[str, Any]) -> AIAgent:
        """Register a new AI agent"""
        agent_id = f"agent-{int(datetime.now().timestamp())}"
        
        agent = AIAgent(
            id=agent_id,
            created_at=datetime.now(),
            updated_at=datetime.now(),
            **agent_data
        )
        
        self.agents[agent_id] = agent
        logger.info(f"Registered AI agent: {agent.name} ({agent.type})")
        return agent

    async def get_agents(self) -> List[AIAgent]:
        """Get all registered agents"""
        return list(self.agents.values())

    async def query_agent(self, query: AgentQuery) -> AgentResponse:
        """Query an AI agent"""
        start_time = datetime.now()
        
        agent = None
        if query.agent_id:
            agent = self.agents.get(query.agent_id)
        else:
            agent = self._get_default_agent()
        
        if not agent:
            raise ValueError("No AI agent available")

        try:
            if agent.type == AgentType.GITHUB_COPILOT:
                response = await self._query_github_copilot(query, agent)
            elif agent.type == AgentType.AMAZON_Q:
                response = await self._query_amazon_q(query, agent)
            else:
                response = await self._query_custom_agent(query, agent)

            processing_time = (datetime.now() - start_time).total_seconds()
            response.processing_time = processing_time
            response.agent_used = agent.name

            return response

        except Exception as e:
            logger.error(f"Error querying agent {agent.name}: {str(e)}")
            raise

    async def _query_github_copilot(self, query: AgentQuery, agent: AIAgent) -> AgentResponse:
        """Query GitHub Copilot"""
        return AgentResponse(
            response=f"GitHub Copilot response for: {query.query}",
            agent_used=agent.name,
            confidence=0.8,
            tokens_used=len(query.query.split()) * 2
        )

    async def _query_amazon_q(self, query: AgentQuery, agent: AIAgent) -> AgentResponse:
        """Query Amazon Q"""
        return AgentResponse(
            response=f"Amazon Q Developer response for: {query.query}",
            agent_used=agent.name,
            confidence=0.85,
            tokens_used=len(query.query.split()) * 2
        )

    async def _query_custom_agent(self, query: AgentQuery, agent: AIAgent) -> AgentResponse:
        """Query custom agent"""
        return AgentResponse(
            response=f"Custom agent response for: {query.query}",
            agent_used=agent.name,
            confidence=0.7,
            tokens_used=len(query.query.split()) * 2
        )

    def _get_default_agent(self) -> Optional[AIAgent]:
        """Get the first connected agent as default"""
        for agent in self.agents.values():
            if agent.status == AgentStatus.CONNECTED:
                return agent
        return None

ai_agent_service = AIAgentService()