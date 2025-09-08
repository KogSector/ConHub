import { Button } from "@/components/ui/button";
import { Mail, Linkedin } from "lucide-react";

export const Footer = () => {
  return (
    <footer className="bg-card border-t border-border">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid md:grid-cols-4 gap-8">
          {/* Brand */}
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <span className="text-xl font-bold text-foreground">ConHub</span>
            </div>
            <p className="text-muted-foreground max-w-xs">
              Unify your repositories with AI for better microservices development.
            </p>
            <div className="flex space-x-3">
              <Button variant="ghost" size="sm" asChild>
                <a href="https://www.linkedin.com/in/rishabh-das-64a336215/" target="_blank" rel="noopener noreferrer">
                  <Linkedin className="w-4 h-4" />
                </a>
              </Button>
              <Button variant="ghost" size="sm" asChild>
                <a href="mailto:rishabh.babi@gmail.com">
                  <Mail className="w-4 h-4" />
                </a>
              </Button>
            </div>
          </div>

          {/* Product */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Product</h3>
            <div className="space-y-2 text-sm">
              <a href="/#features" className="block text-muted-foreground hover:text-foreground transition-colors">
                Features
              </a>
              <a href="/docs" className="block text-muted-foreground hover:text-foreground transition-colors">
                Documentation
              </a>
              <a href="/pricing" className="block text-muted-foreground hover:text-foreground transition-colors">
                Pricing
              </a>
              <a href="/changelog" className="block text-muted-foreground hover:text-foreground transition-colors">
                Changelog
              </a>
            </div>
          </div>

          {/* Developers */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Developers</h3>
            <div className="space-y-2 text-sm">
              <a href="/api" className="block text-muted-foreground hover:text-foreground transition-colors">
                API Reference
              </a>
              <a href="/sdk" className="block text-muted-foreground hover:text-foreground transition-colors">
                SDKs
              </a>
              <a href="/webhooks" className="block text-muted-foreground hover:text-foreground transition-colors">
                Webhooks
              </a>
              <a href="/examples" className="block text-muted-foreground hover:text-foreground transition-colors">
                Examples
              </a>
            </div>
          </div>

          {/* Company */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Company</h3>
            <div className="space-y-2 text-sm">
              <a href="/about" className="block text-muted-foreground hover:text-foreground transition-colors">
                About
              </a>
              <a href="/blog" className="block text-muted-foreground hover:text-foreground transition-colors">
                Blog
              </a>
              <a href="/careers" className="block text-muted-foreground hover:text-foreground transition-colors">
                Careers
              </a>
              <a href="/contact" className="block text-muted-foreground hover:text-foreground transition-colors">
                Contact
              </a>
            </div>
          </div>
        </div>

        <div className="border-t border-border mt-8 pt-8 flex flex-col sm:flex-row justify-between items-center space-y-4 sm:space-y-0">
          <div className="text-sm text-muted-foreground">
            © 2024 ConHub. All rights reserved.
          </div>
        </div>
      </div>
    </footer>
  );
};
