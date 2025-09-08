import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Navbar } from "@/components/ui/navbar";
import { Footer } from "@/components/ui/footer";
import { Check, GitBranch } from "lucide-react";

export default function Pricing() {
  const plans = [
    {
      name: "Free",
      price: "$0",
      period: "forever",
      description: "Perfect for individual developers and small projects",
      features: [
        "Up to 3 repositories",
        "2 AI agent connections",
        "Basic context sharing",
        "Community support",
        "Standard security"
      ],
      popular: false
    },
    {
      name: "Pro",
      price: "$29",
      period: "per month",
      description: "Ideal for growing teams and multiple projects",
      features: [
        "Unlimited repositories",
        "Unlimited AI agents",
        "Advanced context routing",
        "Priority support",
        "Enhanced security",
        "Team collaboration",
        "Custom integrations",
        "Analytics dashboard"
      ],
      popular: true
    },
    {
      name: "Enterprise",
      price: "Custom",
      period: "contact us",
      description: "For large organizations with specific requirements",
      features: [
        "Everything in Pro",
        "SSO integration", 
        "Advanced compliance",
        "Dedicated support",
        "Custom deployment",
        "SLA guarantees",
        "Advanced analytics",
        "White-label options"
      ],
      popular: false
    }
  ];

  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      
      <main className="pt-20">
        {/* Hero Section */}
        <section className="py-24 bg-background">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
            <Badge variant="secondary" className="w-fit mx-auto mb-4">
              Simple Pricing
            </Badge>
            <h1 className="text-4xl md:text-5xl font-bold text-foreground mb-4">
              Choose Your 
              <span className="bg-gradient-to-r from-primary to-primary-glow bg-clip-text text-transparent"> Plan</span>
            </h1>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
              Start free, scale as you grow. No hidden fees, no surprises.
            </p>
          </div>
        </section>

        {/* Pricing Cards */}
        <section className="py-16 -mt-12">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
              {plans.map((plan, index) => (
                <Card 
                  key={index} 
                  className={`relative bg-card border-border ${
                    plan.popular ? 'ring-2 ring-primary shadow-primary' : ''
                  }`}
                >
                  {plan.popular && (
                    <Badge className="absolute -top-3 left-1/2 transform -translate-x-1/2 bg-primary">
                      Most Popular
                    </Badge>
                  )}
                  
                  <CardHeader className="text-center pb-8">
                    <CardTitle className="text-lg font-semibold text-foreground mb-2">
                      {plan.name}
                    </CardTitle>
                    <div className="space-y-1">
                      <div className="text-4xl font-bold text-foreground">
                        {plan.price}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {plan.period}
                      </div>
                    </div>
                    <p className="text-sm text-muted-foreground mt-4">
                      {plan.description}
                    </p>
                  </CardHeader>
                  
                  <CardContent className="space-y-6">
                    <ul className="space-y-3">
                      {plan.features.map((feature, i) => (
                        <li key={i} className="flex items-center text-sm">
                          <Check className="w-4 h-4 text-accent mr-3 flex-shrink-0" />
                          <span className="text-muted-foreground">{feature}</span>
                        </li>
                      ))}
                    </ul>
                    
                    <Button 
                      className={`w-full ${
                        plan.popular 
                          ? 'bg-primary hover:bg-primary/90' 
                          : ''
                      }`}
                      variant={plan.popular ? 'default' : 'outline'}
                    >
                      {plan.name === 'Free' ? 'Get Started Free' : 
                       plan.name === 'Enterprise' ? 'Contact Sales' : 
                       'Start Free Trial'}
                    </Button>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>

        {/* FAQ Section */}
        <section className="py-16 bg-muted/20">
          <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="text-center mb-12">
              <h2 className="text-3xl font-bold text-foreground mb-4">
                Frequently Asked Questions
              </h2>
              <p className="text-muted-foreground">
                Everything you need to know about ConHub pricing
              </p>
            </div>
            
            <div className="grid gap-6">
              {[
                {
                  question: "Can I change plans anytime?",
                  answer: "Yes, you can upgrade or downgrade your plan at any time. Changes take effect immediately."
                },
                {
                  question: "Is there a free trial for paid plans?",
                  answer: "Yes, all paid plans come with a 14-day free trial. No credit card required to start."
                },
                {
                  question: "What happens to my data if I cancel?",
                  answer: "Your data remains accessible for 30 days after cancellation, giving you time to export if needed."
                },
                {
                  question: "Do you offer discounts for students or nonprofits?",
                  answer: "Yes, we offer 50% discounts for students and qualifying nonprofit organizations."
                }
              ].map((faq, index) => (
                <Card key={index} className="bg-card border-border">
                  <CardContent className="p-6">
                    <h3 className="font-semibold text-foreground mb-2">
                      {faq.question}
                    </h3>
                    <p className="text-muted-foreground">
                      {faq.answer}
                    </p>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>
      </main>
      
      <Footer />
    </div>
  );
}