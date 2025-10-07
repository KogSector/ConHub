'use client'

import { useState, useEffect } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { CreditCard, Calendar, TrendingUp, AlertCircle } from 'lucide-react'

interface BillingDashboardData {
  subscription?: {
    subscription: {
      id: string
      status: string
      current_period_end: string
      cancel_at_period_end: boolean
    }
    plan: {
      name: string
      tier: string
      price_monthly: number
      features: Record<string, any>
      limits: Record<string, any>
    }
  }
  payment_methods: Array<{
    id: string
    type: string
    last_four?: string
    brand?: string
    is_default: boolean
  }>
  recent_invoices: Array<{
    id: string
    invoice_number: string
    status: string
    amount_due: number
    due_date?: string
    paid_at?: string
  }>
  usage: Array<{
    resource_type: string
    usage_count: number
    period_start: string
    period_end: string
  }>
}

export function BillingDashboard() {
  const { user, token } = useAuth()
  const [data, setData] = useState<BillingDashboardData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchDashboardData()
  }, [])

  const fetchDashboardData = async () => {
    try {
      const response = await fetch('/api/billing/dashboard', {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      })

      if (response.ok) {
        const dashboardData = await response.json()
        setData(dashboardData)
      } else {
        setError('Failed to load billing data')
      }
    } catch (err) {
      setError('Failed to load billing data')
    } finally {
      setLoading(false)
    }
  }

  const getUsagePercentage = (resourceType: string) => {
    if (!data?.subscription || !data.usage) return 0
    
    const usage = data.usage.find(u => u.resource_type === resourceType)
    const limit = data.subscription.plan.limits[`max_${resourceType}_per_month`]
    
    if (!usage || !limit || limit === null) return 0
    return Math.min((usage.usage_count / limit) * 100, 100)
  }

  const getUsageCount = (resourceType: string) => {
    const usage = data?.usage.find(u => u.resource_type === resourceType)
    return usage?.usage_count || 0
  }

  const getUsageLimit = (resourceType: string) => {
    if (!data?.subscription) return 'N/A'
    const limit = data.subscription.plan.limits[`max_${resourceType}_per_month`]
    return limit === null ? 'Unlimited' : limit
  }

  if (loading) {
    return (
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {[...Array(6)].map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader className="space-y-2">
              <div className="h-4 bg-muted rounded w-3/4"></div>
              <div className="h-3 bg-muted rounded w-1/2"></div>
            </CardHeader>
            <CardContent>
              <div className="h-8 bg-muted rounded"></div>
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5 text-destructive" />
            Error Loading Dashboard
          </CardTitle>
          <CardDescription>{error}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={fetchDashboardData}>Retry</Button>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      {/* Subscription Overview */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Current Plan</CardTitle>
            <CreditCard className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {data?.subscription?.plan.name || 'Free Plan'}
            </div>
            <p className="text-xs text-muted-foreground">
              {data?.subscription ? 
                `$${data.subscription.plan.price_monthly}/month` : 
                'No active subscription'
              }
            </p>
            <Badge 
              variant={data?.subscription?.subscription.status === 'active' ? 'default' : 'secondary'}
              className="mt-2"
            >
              {data?.subscription?.subscription.status || 'free'}
            </Badge>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Next Billing</CardTitle>
            <Calendar className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {data?.subscription ? 
                new Date(data.subscription.subscription.current_period_end).toLocaleDateString() :
                'N/A'
              }
            </div>
            <p className="text-xs text-muted-foreground">
              {data?.subscription?.subscription.cancel_at_period_end ? 
                'Subscription will cancel' : 
                'Next billing date'
              }
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Payment Method</CardTitle>
            <CreditCard className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {data?.payment_methods.find(pm => pm.is_default)?.brand || 'None'}
            </div>
            <p className="text-xs text-muted-foreground">
              {data?.payment_methods.find(pm => pm.is_default)?.last_four ? 
                `•••• ${data.payment_methods.find(pm => pm.is_default)?.last_four}` :
                'No payment method'
              }
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Usage Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Usage This Month
          </CardTitle>
          <CardDescription>
            Track your usage against plan limits
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Repositories</span>
              <span>{getUsageCount('repositories')} / {getUsageLimit('repositories')}</span>
            </div>
            <Progress value={getUsagePercentage('repositories')} className="h-2" />
          </div>

          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>AI Queries</span>
              <span>{getUsageCount('ai_queries')} / {getUsageLimit('ai_queries')}</span>
            </div>
            <Progress value={getUsagePercentage('ai_queries')} className="h-2" />
          </div>

          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Storage (GB)</span>
              <span>{getUsageCount('storage_gb')} / {getUsageLimit('storage_gb')}</span>
            </div>
            <Progress value={getUsagePercentage('storage_gb')} className="h-2" />
          </div>
        </CardContent>
      </Card>

      {/* Recent Invoices */}
      {data?.recent_invoices && data.recent_invoices.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Recent Invoices</CardTitle>
            <CardDescription>Your latest billing history</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {data.recent_invoices.slice(0, 3).map((invoice) => (
                <div key={invoice.id} className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">{invoice.invoice_number}</p>
                    <p className="text-sm text-muted-foreground">
                      {invoice.due_date ? new Date(invoice.due_date).toLocaleDateString() : 'No due date'}
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="font-medium">${invoice.amount_due}</p>
                    <Badge variant={invoice.status === 'paid' ? 'default' : 'secondary'}>
                      {invoice.status}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}