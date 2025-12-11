'use client'

import { useState } from 'react'
import { useAuth } from '@/hooks/use-auth'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { ArrowLeft, CreditCard } from 'lucide-react'
import Link from 'next/link'
import { ProfileAvatar } from '@/components/ui/ProfileAvatar'
import { Footer } from '@/components/ui/footer'
import { BillingDashboard } from '@/components/billing/BillingDashboard'
import { SubscriptionPlans } from '@/components/billing/SubscriptionPlans'
import { PaymentMethods } from '@/components/billing/PaymentMethods'
import { InvoiceHistory } from '@/components/billing/InvoiceHistory'
import { UsageTracking } from '@/components/billing/UsageTracking'
import { StripeProvider } from '@/components/billing/StripeProvider'

export default function BillingPage() {
  const { isAuthenticated } = useAuth()
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
    <StripeProvider>
      <div className="min-h-screen bg-background">
        {/* Header */}
        <div className="border-b border-border bg-card/50 backdrop-blur-sm">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between items-center h-20">
              <div className="flex items-center space-x-4">
                <Link href="/dashboard">
                  <Button variant="ghost" size="sm">
                    <ArrowLeft className="w-4 h-4" />
                  </Button>
                </Link>
                <div className="flex items-center space-x-3">
                  <CreditCard className="w-6 h-6 text-primary" />
                  <h1 className="text-2xl font-bold text-foreground">Billing & Subscription</h1>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <ProfileAvatar />
              </div>
            </div>
          </div>
        </div>

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <p className="text-muted-foreground mb-8">
            Manage your subscription, payment methods, and billing information.
          </p>

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
        
        <Footer />
      </div>
    </StripeProvider>
  )
}
