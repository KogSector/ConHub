'use client'

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Shield, Code, Brain, Database, RefreshCw, GitBranch, Sparkles, Zap, Rocket, Star, ArrowRight, Play } from "lucide-react";
import { Globe } from "@/components/ui/globe";
import { useAuth } from "@/hooks/use-auth";

// Animation helper - animations are always enabled for better UX
const anim = (cls: string) => cls;

export function HeroSection() {
  const { isAuthenticated } = useAuth();
  const delayClasses = [
    'anim-delay-0', 'anim-delay-200', 'anim-delay-400', 'anim-delay-500', 'anim-delay-600',
    'anim-delay-800', 'anim-delay-1000', 'anim-delay-1200', 'anim-delay-1500', 'anim-delay-1700',
    'anim-delay-1900', 'anim-delay-2100', 'anim-delay-2500', 'anim-delay-3000', 'anim-delay-3500',
    'anim-delay-4000', 'anim-delay-4500', 'anim-delay-5000'
  ];
  const durationClasses = [
    'anim-duration-3s', 'anim-duration-4s', 'anim-duration-5s', 'anim-duration-6s', 'anim-duration-7s'
  ];
  
  const handleGetStarted = () => {
    if (isAuthenticated) {
      window.location.href = '/dashboard';
    } else {
      window.location.href = '/auth/login';
    }
  };

  const handleViewDocs = () => {
    window.location.href = "/docs";
  };

  return (
    <section className="min-h-screen flex items-center justify-center bg-background relative overflow-hidden">
      {}
      <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-accent/5" />
      <div className={`absolute top-1/4 -left-1/4 w-96 h-96 bg-primary/10 rounded-full blur-3xl ${anim('animate-pulse')}`} />
      <div className={`absolute bottom-1/4 -right-1/4 w-96 h-96 bg-accent/10 rounded-full blur-3xl ${anim('animate-pulse')} ${anim('anim-delay-1000')}`} />
      <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-gradient-radial from-primary/5 to-transparent rounded-full" />
      
      {}
      <div className="absolute inset-0 overflow-hidden pointer-events-none hero-floating">
        {[...Array(20)].map((_, i) => (
          <div
            key={i}
            className={`absolute w-1 h-1 bg-primary/30 rounded-full dot ${anim('animate-float')} ${anim(delayClasses[Math.floor(Math.random() * delayClasses.length)])} ${anim(durationClasses[Math.floor(Math.random() * durationClasses.length)])} dot-pos-${i}`}
          />
        ))}
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-28 pb-20 relative z-10">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          {}
          <div className={`space-y-8 ${anim('animate-fade-in-up')}`}>
            <div className="flex items-center gap-4">
              <Badge className="w-fit bg-primary/10 text-primary border-primary/20 hover:bg-primary/20 transition-colors">
                <Sparkles className="w-3 h-3 mr-1" />
                Now in Beta
              </Badge>
              <div className="flex items-center gap-1 text-sm text-muted-foreground">
                <Star className="w-4 h-4 text-yellow-500 fill-yellow-500" />
                <span>4.9/5 from early users</span>
              </div>
            </div>
            
            <div className="space-y-6">
              <h1 className="text-4xl md:text-6xl font-bold text-foreground leading-tight">
                <span className={`inline-block ${anim('animate-fade-in-up')}`}>Supercharge Your</span>
                <br />
                <span className={`bg-gradient-to-r from-primary via-primary-glow to-accent bg-clip-text text-transparent inline-block ${anim('animate-fade-in-up')} ${anim('anim-delay-200')}`}>
                  Development
                </span>
                <br />
                <span className={`inline-flex items-center gap-2 ${anim('animate-fade-in-up')} ${anim('anim-delay-400')}`}>
                  with AI
                </span>
              </h1>
              <p className={`text-xl text-muted-foreground leading-relaxed max-w-lg ${anim('animate-fade-in-up')} ${anim('anim-delay-600')}`}>
                Connect repositories, docs, and URLs. Let AI agents access complete context across your entire development ecosystem. 
                <span className="text-primary font-semibold">Code smarter, not harder.</span>
              </p>
            </div>

            {}
            <div className={`grid grid-cols-2 gap-4 ${anim('animate-fade-in-up')} ${anim('anim-delay-800')}`}>
              <div className="flex items-center gap-2 p-3 rounded-lg bg-card/50 border border-border/50 hover:bg-card transition-colors">
                <Shield className="w-5 h-5 text-accent" />
                <span className="text-sm font-medium">Enterprise Security</span>
              </div>
              <div className="flex items-center gap-2 p-3 rounded-lg bg-card/50 border border-border/50 hover:bg-card transition-colors">
                <Code className="w-5 h-5 text-primary" />
                <span className="text-sm font-medium">Multi-source Context</span>
              </div>
              <div className="flex items-center gap-2 p-3 rounded-lg bg-card/50 border border-border/50 hover:bg-card transition-colors">
                <Brain className="w-5 h-5 text-primary-glow" />
                <span className="text-sm font-medium">AI Powered</span>
              </div>
              <div className="flex items-center gap-2 p-3 rounded-lg bg-card/50 border border-border/50 hover:bg-card transition-colors">
                <Database className="w-5 h-5 text-accent" />
                <span className="text-sm font-medium">RAG Enabled</span>
              </div>
            </div>

            {}
            <div className={`flex flex-col sm:flex-row gap-4 ${anim('animate-fade-in-up')} ${anim('anim-delay-1000')}`}>
              <Button 
                className="h-11 px-8 bg-gradient-to-r from-primary to-primary-glow hover:from-primary/90 hover:to-primary-glow/90 transition-all duration-300 shadow-lg hover:shadow-xl group" 
                onClick={handleGetStarted}
              >
                <Rocket className="w-4 h-4 mr-2 group-hover:animate-bounce" />
                Get Started
                <ArrowRight className="w-4 h-4 ml-2 group-hover:translate-x-1 transition-transform" />
              </Button>
              <Button 
                className="h-11 px-8 border bg-background text-foreground hover:bg-accent hover:text-accent-foreground border-primary/20 hover:border-primary/40 transition-all duration-300 group" 
                onClick={handleViewDocs}
              >
                <Play className="w-4 h-4 mr-2" />
                Watch Demo
              </Button>
            </div>

            {}
            <div className={`flex items-center gap-8 text-sm ${anim('animate-fade-in-up')} ${anim('anim-delay-1200')}`}>
              <div className="text-center">
                <div className="font-bold text-2xl bg-gradient-to-r from-primary to-primary-glow bg-clip-text text-transparent">1000+</div>
                <div className="text-muted-foreground">Sources Connected</div>
              </div>
              <div className="text-center">
                <div className="font-bold text-2xl bg-gradient-to-r from-accent to-primary bg-clip-text text-transparent">50+</div>
                <div className="text-muted-foreground">AI Agents</div>
              </div>
              <div className="text-center">
                <div className="font-bold text-2xl bg-gradient-to-r from-green-500 to-primary bg-clip-text text-transparent">99.9%</div>
                <div className="text-muted-foreground">Uptime</div>
              </div>
            </div>
          </div>

          {}
          <div className={`relative ${anim('animate-fade-in-up')} ${anim('anim-delay-500')}`}>
            <div className="relative rounded-2xl overflow-hidden shadow-2xl">
              <div className="relative">
                <Globe className="w-full h-auto" />
                {}
                <div className="absolute inset-0 bg-gradient-to-r from-primary/20 to-accent/20 rounded-2xl blur-xl -z-10" />
              </div>
            </div>
            
            {}
            <div className={`absolute -top-6 -right-6 bg-gradient-to-r from-card to-card/90 backdrop-blur-sm border border-border/50 rounded-xl p-4 shadow-xl ${anim('animate-fade-in-up')} ${anim('anim-delay-1500')}`}>
              <div className="flex items-center gap-3">
                <div className={`w-3 h-3 bg-green-500 rounded-full ${anim('animate-pulse')}`} />
                <div>
                  <div className="text-sm font-semibold text-foreground">5 AI Agents</div>
                  <div className="text-xs text-muted-foreground">Connected & Active</div>
                </div>
              </div>
            </div>
            
            <div className={`absolute -bottom-6 -left-6 bg-gradient-to-r from-card to-card/90 backdrop-blur-sm border border-border/50 rounded-xl p-4 shadow-xl ${anim('animate-fade-in-up')} ${anim('anim-delay-1700')}`}>
              <div className="flex items-center gap-3">
                <RefreshCw className="w-5 h-5 text-primary animate-spin anim-duration-3s" />
                <div>
                  <div className="text-sm font-semibold text-foreground">12 Sources</div>
                  <div className="text-xs text-muted-foreground">Syncing in real-time</div>
                </div>
              </div>
            </div>

            <div className={`absolute top-1/2 -left-10 bg-gradient-to-r from-card to-card/90 backdrop-blur-sm border border-border/50 rounded-xl p-4 shadow-xl ${anim('animate-fade-in-up')} ${anim('anim-delay-1900')}`}>
              <div className="flex items-center gap-3">
                <GitBranch className="w-5 h-5 text-green-500" />
                <div>
                  <div className="text-sm font-semibold text-foreground">3 Repositories</div>
                  <div className="text-xs text-muted-foreground">Being analyzed</div>
                </div>
              </div>
            </div>

            {}
            <div className={`absolute top-8 left-8 bg-gradient-to-r from-card to-card/90 backdrop-blur-sm border border-border/50 rounded-xl p-3 shadow-xl ${anim('animate-fade-in-up')} ${anim('anim-delay-2100')}`}>
              <div className="flex items-center gap-2">
                <Zap className="w-4 h-4 text-yellow-500" />
                <span className="text-xs font-medium text-foreground">Lightning Fast</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
