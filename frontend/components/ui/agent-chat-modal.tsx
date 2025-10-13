"use client";

import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useToast } from "@/hooks/use-toast";
import { apiClient, AgentRecord, AgentInvokeRequest, AgentInvokeResponse } from "@/lib/api";
import { Bot, User, Send, Loader2, RefreshCw, Clock, Zap } from "lucide-react";

interface AgentChatModalProps {
  agent: AgentRecord;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface ChatMessage {
  id: string;
  type: 'user' | 'agent';
  content: string;
  timestamp: Date;
  response_time?: number;
  tokens_used?: number;
  context_used?: string[];
}

const CONTEXT_OPTIONS = [
  { value: "all", label: "All Available Context" },
  { value: "repositories", label: "Repositories Only" },
  { value: "documents", label: "Documents Only" },
  { value: "urls", label: "URLs Only" },
  { value: "none", label: "No Context" },
];

export function AgentChatModal({ agent, open, onOpenChange }: AgentChatModalProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [contextType, setContextType] = useState("all");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { toast } = useToast();

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(scrollToBottom, [messages]);

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      type: 'user',
      content: input.trim(),
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInput("");
    setIsLoading(true);

    try {
      const request: AgentInvokeRequest = {
        message: userMessage.content,
        context_type: contextType === "none" ? undefined : contextType,
        include_history: true,
      };

      const result = await apiClient.invokeAgent(agent.id, request);

      if (result.success && result.data) {
        const agentMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          type: 'agent',
          content: result.data.response,
          timestamp: new Date(),
          response_time: result.data.usage.response_time_ms,
          tokens_used: result.data.usage.tokens_used,
          context_used: result.data.context_used,
        };

        setMessages(prev => [...prev, agentMessage]);
      } else {
        toast({
          title: "Error",
          description: result.message || "Failed to get response from agent",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to communicate with agent",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const clearChat = () => {
    setMessages([]);
  };

  const getStatusColor = (status: AgentRecord['status']) => {
    switch (status) {
      case 'Connected':
        return 'bg-green-100 text-green-800';
      case 'Pending':
        return 'bg-yellow-100 text-yellow-800';
      case 'Error':
        return 'bg-red-100 text-red-800';
      case 'Inactive':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl h-[80vh] flex flex-col">
        <DialogHeader className="flex-shrink-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Bot className="w-5 h-5" />
              <DialogTitle>Chat with {agent.name}</DialogTitle>
              <Badge className={getStatusColor(agent.status)}>
                {agent.status}
              </Badge>
              <Badge variant="outline">{agent.agent_type}</Badge>
            </div>
            <Button variant="outline" size="sm" onClick={clearChat}>
              <RefreshCw className="w-4 h-4 mr-2" />
              Clear
            </Button>
          </div>
        </DialogHeader>

        {}
        <div className="flex-shrink-0 border-b pb-4">
          <div className="flex items-center gap-4">
            <label htmlFor="context-select" className="text-sm font-medium">
              Context:
            </label>
            <Select value={contextType} onValueChange={setContextType}>
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CONTEXT_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        {}
        <ScrollArea className="flex-1 pr-4">
          <div className="space-y-4 py-4">
            {messages.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                <Bot className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
                <p>Start a conversation with {agent.name}</p>
                <p className="text-sm">
                  This agent has access to: {agent.permissions.join(", ")}
                </p>
              </div>
            ) : (
              messages.map((message) => (
                <div
                  key={message.id}
                  className={`flex gap-3 ${
                    message.type === 'user' ? 'justify-end' : 'justify-start'
                  }`}
                >
                  {message.type === 'agent' && (
                    <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                      <Bot className="w-4 h-4" />
                    </div>
                  )}
                  
                  <div
                    className={`max-w-[70%] rounded-lg px-4 py-2 ${
                      message.type === 'user'
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted'
                    }`}
                  >
                    <div className="text-sm whitespace-pre-wrap">
                      {message.content}
                    </div>
                    
                    <div className="flex items-center gap-2 mt-2 text-xs opacity-70">
                      <span>{message.timestamp.toLocaleTimeString()}</span>
                      
                      {message.type === 'agent' && (
                        <>
                          {message.response_time && (
                            <div className="flex items-center gap-1">
                              <Clock className="w-3 h-3" />
                              <span>{message.response_time}ms</span>
                            </div>
                          )}
                          
                          {message.tokens_used && (
                            <div className="flex items-center gap-1">
                              <Zap className="w-3 h-3" />
                              <span>{message.tokens_used} tokens</span>
                            </div>
                          )}
                          
                          {message.context_used && message.context_used.length > 0 && (
                            <div className="flex gap-1">
                              {message.context_used.map((context) => (
                                <Badge key={context} variant="secondary" className="text-xs">
                                  {context}
                                </Badge>
                              ))}
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  </div>

                  {message.type === 'user' && (
                    <div className="w-8 h-8 rounded-full bg-secondary flex items-center justify-center flex-shrink-0">
                      <User className="w-4 h-4" />
                    </div>
                  )}
                </div>
              ))
            )}
            
            {isLoading && (
              <div className="flex gap-3 justify-start">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                  <Bot className="w-4 h-4" />
                </div>
                <div className="bg-muted rounded-lg px-4 py-2">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    {agent.name} is thinking...
                  </div>
                </div>
              </div>
            )}
            
            <div ref={messagesEndRef} />
          </div>
        </ScrollArea>

        {}
        <form onSubmit={handleSendMessage} className="flex-shrink-0 border-t pt-4">
          <div className="flex gap-2">
            <Input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder={`Message ${agent.name}...`}
              disabled={isLoading || agent.status !== 'Connected'}
              className="flex-1"
              maxLength={2000}
            />
            <Button 
              type="submit" 
              disabled={!input.trim() || isLoading || agent.status !== 'Connected'}
              size="icon"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Send className="w-4 h-4" />
              )}
            </Button>
          </div>
          
          {agent.status !== 'Connected' && (
            <p className="text-sm text-muted-foreground mt-2">
              Agent must be connected to send messages. Current status: {agent.status}
            </p>
          )}
        </form>
      </DialogContent>
    </Dialog>
  );
}