'use client'

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Shield, Code, GitBranch, Brain, TestTube2, Database, RefreshCw } from "lucide-react";
import Image from "next/image";
import { useAuth } from "@/hooks/use-auth";


export const HeroSection = () => {
  const { isAuthenticated, login } = useAuth();
  
  const handleGetStarted = () => {
    if (isAuthenticated) {
      window.location.href = '/dashboard';
    } else {
      login();
    }
  };

  const handleViewDocs = () => {
    window.location.href = "/docs";
  };

  return (
    <section className="min-h-screen flex items-center justify-center bg-background relative overflow-hidden">
      {/* Background decoration */}
      <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-accent/5" />
      <div className="absolute top-1/4 -left-1/4 w-96 h-96 bg-primary/10 rounded-full blur-3xl" />
      <div className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-accent/10 rounded-full blur-3xl" />

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-28 pb-20 relative z-10">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          {/* Left Column - Content */}
          <div className="space-y-8">
            <Badge variant="secondary" className="w-fit">
              <TestTube2 className="w-3 h-3 mr-1" />
              Now in Beta
            </Badge>
            
            <div className="space-y-4">
              <h1 className="text-4xl md:text-6xl font-bold text-foreground leading-tight">
                Supercharge Your 
                <span className="bg-gradient-to-r from-primary to-primary-glow bg-clip-text text-transparent"> Development</span>
                {" "}with AI
              </h1>
              <p className="text-xl text-muted-foreground leading-relaxed max-w-lg">
                Connect repositories, docs, and URLs. Let AI agents access complete context across your entire development ecosystem. Code smarter, not harder.
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm text-muted-foreground">
              <div className="flex items-center gap-1">
                <Shield className="w-4 h-4 text-accent" />
                <span>Secure by design</span>
              </div>
              <div className="flex items-center gap-1">
                <Code className="w-4 h-4 text-primary" />
                <span>Multi-source context</span>
              </div>
              <div className="flex items-center gap-1">
                <Brain className="w-4 h-4 text-primary-glow" />
                <span>AI powered</span>
              </div>
              <div className="flex items-center gap-1">
                <Database className="w-4 h-4 text-accent" />
                <span>RAG enabled</span>
              </div>
            </div>

            <div className="flex flex-col sm:flex-row gap-4">
              <Button size="lg" className="bg-primary hover:bg-primary/90 transition-all duration-300" onClick={handleGetStarted}>
                Get Started
              </Button>
              <Button variant="outline" size="lg" onClick={handleViewDocs}>
                View Documentation
              </Button>
            </div>

            <div className="flex items-center gap-8 text-sm text-muted-foreground">
              <div>
                <div className="font-semibold text-foreground">1000+</div>
                <div>Sources connected</div>
              </div>
              <div>
                <div className="font-semibold text-foreground">50+</div>
                <div>AI agents integrated</div>
              </div>
              <div>
                <div className="font-semibold text-foreground">99.9%</div>
                <div>Uptime guaranteed</div>
              </div>
            </div>
          </div>

          {/* Right Column - Hero Image */}
          <div className="relative">
            <div className="relative rounded-2xl overflow-hidden shadow-glow">
              <Image
                src="/assets/hero-image.jpg"
                alt="ConHub Dashboard showing connected sources and AI agents"
                className="w-full h-auto object-cover"
                width={600}
                height={400}
                priority
              />
              <div className="absolute inset-0 bg-gradient-to-t from-background/20 to-transparent" />
            </div>
            
            {/* Floating elements */}
            <div className="absolute -top-4 -right-4 bg-card border border-border rounded-lg p-3 shadow-card">
              <div className="flex items-center gap-2 text-sm">
                <div className="w-2 h-2 bg-accent rounded-full animate-pulse" />
                <span className="text-muted-foreground">5 AI agents connected</span>
              </div>
            </div>
            
            <div className="absolute -bottom-4 -left-4 bg-card border border-border rounded-lg p-3 shadow-card">
              <div className="flex items-center gap-2 text-sm">
                <RefreshCw className="w-4 h-4 text-primary" />
                <span className="text-muted-foreground">12 sources synced</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
