'use client'

import { useState } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { BillingDashboard } from '@/components/billing/billing-dashboard'
import { SubscriptionPlans } from '@/components/billing/subscription-plans'
import { PaymentMethods } from '@/components/billing/payment-methods'
import { InvoiceHistory } from '@/components/billing/invoice-history'
import { UsageTracking } from '@/components/billing/usage-tracking'

export default function BillingPage() {
  const { user, isAuthenticated } = useAuth()
  const [activeTab, setActiveTab] = useState('dashboard')

  if (!isAuthenticated) {
    return (
      <div className="container mx-auto px-4 py-8">
        <Card>
          <CardHeader>
            <CardTitle>Authentication Required</CardTitle>
            <CardDescription>
              Please log in to access your billing information.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Billing & Subscription</h1>
        <p className="text-muted-foreground mt-2">
          Manage your subscription, payment methods, and billing information.
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="dashboard">Dashboard</TabsTrigger>
          <TabsTrigger value="plans">Plans</TabsTrigger>
          <TabsTrigger value="payment">Payment</TabsTrigger>
          <TabsTrigger value="invoices">Invoices</TabsTrigger>
          <TabsTrigger value="usage">Usage</TabsTrigger>
        </TabsList>

        <TabsContent value="dashboard">
          <BillingDashboard />
        </TabsContent>

        <TabsContent value="plans">
          <SubscriptionPlans />
        </TabsContent>

        <TabsContent value="payment">
          <PaymentMethods />
        </TabsContent>

        <TabsContent value="invoices">
          <InvoiceHistory />
        </TabsContent>

        <TabsContent value="usage">
          <UsageTracking />
        </TabsContent>
      </Tabs>
    </div>
  )
}