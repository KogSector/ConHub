'use client'

import { useState, useEffect } from 'react'
import { useAuth } from '@/hooks/use-auth'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line } from 'recharts'
import { TrendingUp, Database, Zap, FolderOpen, AlertTriangle } from 'lucide-react'

interface UsageData {
  resource_type: string
  usage_count: number
  period_start: string
  period_end: string
}

interface UsageSummary {
  repositories: number
  ai_queries: number
  storage_gb: number
  limits: Record<string, any>
}

export function UsageTracking() {
  const { token } = useAuth()
  const [usageData, setUsageData] = useState<UsageData[]>([])
  const [currentSubscription, setCurrentSubscription] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [selectedPeriod, setSelectedPeriod] = useState('current')

  useEffect(() => {
    fetchUsageData()
    fetchCurrentSubscription()
  }, [selectedPeriod])

  const fetchUsageData = async () => {
    try {
      const periodStart = getPeriodStart(selectedPeriod)
      const headers: Record<string, string> = {}
      if (token) headers['Authorization'] = `Bearer ${token}`
      const response = await fetch(`/api/billing/dashboard`, { headers })
      if (response.ok) {
        const data = await response.json()
        setUsageData(data.usage || [])
      }
    } catch (error) {
      console.error('Failed to fetch usage data:', error)
    } finally {
      setLoading(false)
    }
  }

  const fetchCurrentSubscription = async () => {
    try {
      const headers: Record<string, string> = {}
      if (token) headers['Authorization'] = `Bearer ${token}`
      const response = await fetch('/api/billing/subscription', { headers })
      if (response.ok) {
        const subscription = await response.json()
        setCurrentSubscription(subscription)
      }
    } catch (error) {
      console.error('Failed to fetch subscription:', error)
    }
  }

  const getPeriodStart = (period: string) => {
    const now = new Date()
    switch (period) {
      case 'current':
        return new Date(now.getFullYear(), now.getMonth(), 1)
      case 'last':
        return new Date(now.getFullYear(), now.getMonth() - 1, 1)
      case '3months':
        return new Date(now.getFullYear(), now.getMonth() - 3, 1)
      default:
        return new Date(now.getFullYear(), now.getMonth(), 1)
    }
  }

  const getUsageByType = (resourceType: string) => {
    const usage = usageData.find(u => u.resource_type === resourceType)
    return usage?.usage_count || 0
  }

  const getLimit = (resourceType: string) => {
    if (!currentSubscription?.plan?.limits) return null
    const limit = currentSubscription.plan.limits[`max_${resourceType}_per_month`]
    return limit === null ? 'Unlimited' : limit
  }

  const getUsagePercentage = (resourceType: string) => {
    const usage = getUsageByType(resourceType)
    const limit = getLimit(resourceType)
    
    if (limit === 'Unlimited' || limit === null) return 0
    return Math.min((usage / limit) * 100, 100)
  }

  const isNearLimit = (resourceType: string) => {
    const percentage = getUsagePercentage(resourceType)
    return percentage >= 80
  }

  const getResourceIcon = (resourceType: string) => {
    switch (resourceType) {
      case 'ai_queries':
        return <Zap className="h-5 w-5" />
      case 'storage_gb':
        return <Database className="h-5 w-5" />
      default:
        return <TrendingUp className="h-5 w-5" />
    }
  }

  const getResourceLabel = (resourceType: string) => {
    switch (resourceType) {
      case 'ai_queries':
        return 'Requests'
      case 'storage_gb':
        return 'Storage (GB)'
      default:
        return resourceType.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())
    }
  }

  const chartData = usageData.map(usage => ({
    name: getResourceLabel(usage.resource_type),
    usage: usage.usage_count,
    limit: getLimit(usage.resource_type) === 'Unlimited' ? null : getLimit(usage.resource_type),
  }))

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="grid gap-6 md:grid-cols-3">
          {[...Array(3)].map((_, i) => (
            <Card key={i} className="animate-pulse">
              <CardHeader className="space-y-2">
                <div className="h-4 bg-muted rounded w-3/4"></div>
                <div className="h-8 bg-muted rounded w-1/2"></div>
              </CardHeader>
              <CardContent>
                <div className="h-2 bg-muted rounded"></div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Usage Tracking</h2>
          <p className="text-muted-foreground">
            Monitor your resource usage and plan limits
          </p>
        </div>
        <Select value={selectedPeriod} onValueChange={setSelectedPeriod}>
          <SelectTrigger className="w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="current">Current Month</SelectItem>
            <SelectItem value="last">Last Month</SelectItem>
            <SelectItem value="3months">Last 3 Months</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {}
      <div className="grid gap-6 md:grid-cols-2">
        {['ai_queries', 'storage_gb'].map((resourceType) => (
          <Card key={resourceType}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium flex items-center gap-2">
                {getResourceIcon(resourceType)}
                {getResourceLabel(resourceType)}
              </CardTitle>
              {isNearLimit(resourceType) && (
                <AlertTriangle className="h-4 w-4 text-orange-500" />
              )}
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {getUsageByType(resourceType).toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">
                of {getLimit(resourceType) === 'Unlimited' ? 'unlimited' : getLimit(resourceType)?.toLocaleString()}
              </p>
              {getLimit(resourceType) !== 'Unlimited' && (
                <div className="mt-3">
                  <Progress 
                    value={getUsagePercentage(resourceType)} 
                    className={`h-2 ${isNearLimit(resourceType) ? 'bg-orange-100' : ''}`}
                  />
                  <div className="flex justify-between text-xs text-muted-foreground mt-1">
                    <span>{getUsagePercentage(resourceType).toFixed(1)}% used</span>
                    {isNearLimit(resourceType) && (
                      <Badge variant="outline" className="text-orange-600 border-orange-200">
                        Near Limit
                      </Badge>
                    )}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {}
      {chartData.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Usage Overview
            </CardTitle>
            <CardDescription>
              Visual representation of your current usage vs limits
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-80">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="name" />
                  <YAxis />
                  <Tooltip />
                  <Bar dataKey="usage" fill="#3b82f6" name="Current Usage" />
                  <Bar dataKey="limit" fill="#e5e7eb" name="Limit" />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>
      )}

      {}
      {currentSubscription && (
        <Card>
          <CardHeader>
            <CardTitle>Current Plan: {currentSubscription.plan.name}</CardTitle>
            <CardDescription>
              Your plan limits and features
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-2">
              <div>
                <h4 className="font-semibold mb-2">Plan Limits</h4>
                <ul className="space-y-1 text-sm">
                  <li>Repositories: {getLimit('repositories') === 'Unlimited' ? 'Unlimited' : getLimit('repositories')}</li>
                  <li>AI Queries: {getLimit('ai_queries') === 'Unlimited' ? 'Unlimited' : `${getLimit('ai_queries')}/month`}</li>
                  <li>Storage: {getLimit('storage_gb') === 'Unlimited' ? 'Unlimited' : `${getLimit('storage_gb')} GB`}</li>
                </ul>
              </div>
              <div>
                <h4 className="font-semibold mb-2">Plan Features</h4>
                <div className="flex flex-wrap gap-1">
                  {Object.entries(currentSubscription.plan.features as Record<string, boolean>).map(([key, value]) => (
                    value
                      ? (
                          <Badge key={key} variant="secondary" className="text-xs">
                            {key.replace('_', ' ')}
                          </Badge>
                        )
                      : null
                  ))}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}