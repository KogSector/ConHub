import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { 
  CreditCard, 
  Download, 
  Calendar,
  TrendingUp,
  Package,
  Receipt,
  ExternalLink,
  AlertCircle
} from "lucide-react";

export function BillingSettings() {
  const invoices = [
    {
      id: "INV-2024-001",
      date: "2024-09-01",
      amount: "$29.99",
      status: "paid",
      downloadUrl: "#"
    },
    {
      id: "INV-2024-002",
      date: "2024-08-01",
      amount: "$29.99",
      status: "paid",
      downloadUrl: "#"
    },
    {
      id: "INV-2024-003",
      date: "2024-07-01",
      amount: "$29.99",
      status: "paid",
      downloadUrl: "#"
    },
    {
      id: "INV-2024-004",
      date: "2024-06-01",
      amount: "$29.99",
      status: "paid",
      downloadUrl: "#"
    }
  ];

  return (
    <div className="space-y-6">
      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Package className="w-5 h-5" />
            Current Plan
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 rounded-lg bg-gradient-to-r from-primary/10 to-accent/10 border border-primary/20">
            <div>
              <div className="flex items-center gap-2 mb-1">
                <h3 className="text-lg font-semibold">Pro Plan</h3>
                <Badge variant="default">Current</Badge>
              </div>
              <p className="text-sm text-muted-foreground">$29.99/month • Billed monthly</p>
              <p className="text-sm text-muted-foreground">Next billing: October 1, 2024</p>
            </div>
            <div className="text-right">
              <div className="text-2xl font-bold">$29.99</div>
              <div className="text-sm text-muted-foreground">/month</div>
            </div>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>Repositories</span>
                <span>12 / 50</span>
              </div>
              <Progress value={24} className="h-2" />
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>AI Requests</span>
                <span>2,773 / 10,000</span>
              </div>
              <Progress value={27.73} className="h-2" />
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>Team Members</span>
                <span>3 / 10</span>
              </div>
              <Progress value={30} className="h-2" />
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>Storage</span>
                <span>2.1 GB / 100 GB</span>
              </div>
              <Progress value={2.1} className="h-2" />
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CreditCard className="w-5 h-5" />
            Payment Method
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 rounded-lg border border-border bg-muted/20">
            <div className="flex items-center gap-3">
              <div className="w-10 h-6 bg-gradient-to-r from-blue-500 to-purple-500 rounded flex items-center justify-center">
                <span className="text-xs font-bold text-white">VISA</span>
              </div>
              <div>
                <div className="font-medium">•••• •••• •••• 4242</div>
                <div className="text-sm text-muted-foreground">Expires 12/2026</div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant="default">Primary</Badge>
              <Button variant="outline" size="sm">Update</Button>
            </div>
          </div>

          <Button variant="outline" className="w-full">
            <CreditCard className="w-4 h-4 mr-2" />
            Add Payment Method
          </Button>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="w-5 h-5" />
            Usage This Month
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-3">
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-primary">2,773</div>
              <div className="text-sm text-muted-foreground">AI Requests</div>
              <div className="text-xs text-muted-foreground">27.7% of limit</div>
            </div>
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-accent">12</div>
              <div className="text-sm text-muted-foreground">Repositories</div>
              <div className="text-xs text-muted-foreground">24% of limit</div>
            </div>
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-primary-glow">2.1 GB</div>
              <div className="text-sm text-muted-foreground">Storage Used</div>
              <div className="text-xs text-muted-foreground">2.1% of limit</div>
            </div>
          </div>

          <div className="p-4 rounded-lg bg-yellow-50 dark:bg-yellow-950 border border-yellow-200 dark:border-yellow-800">
            <div className="flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-yellow-600" />
              <span className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                Usage Alert
              </span>
            </div>
            <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
              You&apos;re approaching your AI request limit. Consider upgrading to avoid service interruption.
            </p>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Receipt className="w-5 h-5" />
              Billing History
            </CardTitle>
            <Button variant="outline" size="sm">
              <Download className="w-4 h-4 mr-2" />
              Download All
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {invoices.map((invoice) => (
              <div key={invoice.id} className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/20">
                <div className="flex items-center gap-3">
                  <Receipt className="w-4 h-4 text-muted-foreground" />
                  <div>
                    <div className="font-medium">{invoice.id}</div>
                    <div className="text-sm text-muted-foreground flex items-center gap-2">
                      <Calendar className="w-3 h-3" />
                      {invoice.date}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <div className="text-right">
                    <div className="font-medium">{invoice.amount}</div>
                    <Badge 
                      variant={invoice.status === "paid" ? "default" : "secondary"}
                      className="text-xs"
                    >
                      {invoice.status}
                    </Badge>
                  </div>
                  <Button variant="ghost" size="sm">
                    <Download className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Package className="w-5 h-5" />
            Available Plans
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            {}
            <div className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-center mb-4">
                <h3 className="text-lg font-semibold">Free</h3>
                <div className="text-2xl font-bold">$0</div>
                <div className="text-sm text-muted-foreground">/month</div>
              </div>
              <ul className="space-y-2 text-sm">
                <li>• 3 repositories</li>
                <li>• 100 AI requests/month</li>
                <li>• 1 team member</li>
                <li>• 1 GB storage</li>
                <li>• Community support</li>
              </ul>
              <Button variant="outline" className="w-full mt-4" disabled>
                Current Plan
              </Button>
            </div>

            {}
            <div className="p-4 rounded-lg border-2 border-primary bg-primary/5">
              <div className="text-center mb-4">
                <div className="flex items-center justify-center gap-2">
                  <h3 className="text-lg font-semibold">Pro</h3>
                  <Badge variant="default">Current</Badge>
                </div>
                <div className="text-2xl font-bold">$29.99</div>
                <div className="text-sm text-muted-foreground">/month</div>
              </div>
              <ul className="space-y-2 text-sm">
                <li>• 50 repositories</li>
                <li>• 10,000 AI requests/month</li>
                <li>• 10 team members</li>
                <li>• 100 GB storage</li>
                <li>• Priority support</li>
                <li>• Advanced analytics</li>
              </ul>
              <Button className="w-full mt-4" disabled>
                Current Plan
              </Button>
            </div>

            {}
            <div className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-center mb-4">
                <h3 className="text-lg font-semibold">Enterprise</h3>
                <div className="text-2xl font-bold">$99.99</div>
                <div className="text-sm text-muted-foreground">/month</div>
              </div>
              <ul className="space-y-2 text-sm">
                <li>• Unlimited repositories</li>
                <li>• Unlimited AI requests</li>
                <li>• Unlimited team members</li>
                <li>• 1 TB storage</li>
                <li>• 24/7 phone support</li>
                <li>• Custom integrations</li>
                <li>• SLA guarantee</li>
              </ul>
              <Button variant="outline" className="w-full mt-4">
                <ExternalLink className="w-4 h-4 mr-2" />
                Contact Sales
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle>Billing Preferences</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <label className="text-sm font-medium">Billing cycle</label>
              <p className="text-sm text-muted-foreground">
                Change between monthly and annual billing
              </p>
            </div>
            <Button variant="outline">
              Switch to Annual (Save 20%)
            </Button>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <label className="text-sm font-medium">Invoice notifications</label>
              <p className="text-sm text-muted-foreground">
                Get notified when new invoices are available
              </p>
            </div>
            <Button variant="outline">Configure</Button>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle>Plan Management</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <Button variant="outline">
              <TrendingUp className="w-4 h-4 mr-2" />
              Upgrade Plan
            </Button>
            <Button variant="outline">
              View All Plans
            </Button>
            <Button variant="outline" className="text-destructive">
              Cancel Subscription
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
