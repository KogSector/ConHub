'use client'

import { useState, useEffect } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Check, Star, Zap, Building } from 'lucide-react'
import { useToast } from '@/hooks/use-toast'

interface SubscriptionPlan {
  id: string
  name: string
  description: string
  tier: string
  price_monthly: number
  price_yearly?: number
  features: Record<string, any>
  limits: Record<string, any>
  is_active: boolean
}

export function SubscriptionPlans() {
  const { user, token } = useAuth()
  const { toast } = useToast()
  const [plans, setPlans] = useState<SubscriptionPlan[]>([])
  const [currentSubscription, setCurrentSubscription] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [subscribing, setSubscribing] = useState<string | null>(null)

  useEffect(() => {
    fetchPlans()
    fetchCurrentSubscription()
  }, [])

  const fetchPlans = async () => {
    try {
      const response = await fetch('/api/billing/plans')
      if (response.ok) {
        const plansData = await response.json()
        setPlans(plansData)
      }
    } catch (error) {
      console.error('Failed to fetch plans:', error)
    }
  }

  const fetchCurrentSubscription = async () => {
    try {
      const response = await fetch('/api/billing/subscription', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      })
      if (response.ok) {
        const subscription = await response.json()
        setCurrentSubscription(subscription)
      }
    } catch (error) {
      console.error('Failed to fetch subscription:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleSubscribe = async (planId: string) => {
    setSubscribing(planId)
    try {
      const response = await fetch('/api/billing/subscription', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ plan_id: planId }),
      })

      if (response.ok) {
        toast({
          title: 'Subscription Updated',
          description: 'Your subscription has been updated successfully.',
        })
        fetchCurrentSubscription()
      } else {
        const error = await response.json()
        toast({
          title: 'Subscription Failed',
          description: error.error || 'Failed to update subscription',
          variant: 'destructive',
        })
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      })
    } finally {
      setSubscribing(null)
    }
  }

  const getPlanIcon = (tier: string) => {
    switch (tier) {
      case 'free':
        return <Star className="h-6 w-6" />
      case 'personal':
        return <Zap className="h-6 w-6" />
      case 'team':
        return <Building className="h-6 w-6" />
      case 'enterprise':
        return <Building className="h-6 w-6 text-purple-500" />
      default:
        return <Star className="h-6 w-6" />
    }
  }

  const isCurrentPlan = (planId: string) => {
    return currentSubscription?.plan?.id === planId
  }

  const getFeatureList = (features: Record<string, any>) => {
    const featureList = []
    
    if (features.repositories) {
      featureList.push(`${features.repositories === 'unlimited' ? 'Unlimited' : features.repositories} repositories`)
    }
    if (features.ai_queries) {
      featureList.push(`${features.ai_queries === 'unlimited' ? 'Unlimited' : features.ai_queries} AI queries/month`)
    }
    if (features.storage_gb) {
      featureList.push(`${features.storage_gb}GB storage`)
    }
    if (features.support) {
      featureList.push(`${features.support} support`)
    }
    if (features.advanced_search) {
      featureList.push('Advanced search')
    }
    if (features.team_collaboration) {
      featureList.push('Team collaboration')
    }
    if (features.github_apps) {
      featureList.push('GitHub Apps integration')
    }
    if (features.sso) {
      featureList.push('Single Sign-On (SSO)')
    }
    if (features.audit_logs) {
      featureList.push('Audit logs')
    }
    if (features.custom_integrations) {
      featureList.push('Custom integrations')
    }

    return featureList
  }

  if (loading) {
    return (
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        {[...Array(4)].map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader className="space-y-2">
              <div className="h-6 bg-muted rounded"></div>
              <div className="h-4 bg-muted rounded w-3/4"></div>
              <div className="h-8 bg-muted rounded w-1/2"></div>
            </CardHeader>
            <CardContent className="space-y-2">
              {[...Array(5)].map((_, j) => (
                <div key={j} className="h-4 bg-muted rounded"></div>
              ))}
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold">Choose Your Plan</h2>
        <p className="text-muted-foreground mt-2">
          Select the perfect plan for your development needs
        </p>
      </div>

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        {plans.map((plan) => (
          <Card 
            key={plan.id} 
            className={`relative ${
              plan.tier === 'team' ? 'border-primary shadow-lg' : ''
            } ${isCurrentPlan(plan.id) ? 'ring-2 ring-primary' : ''}`}
          >
            {plan.tier === 'team' && (
              <Badge className="absolute -top-2 left-1/2 transform -translate-x-1/2">
                Most Popular
              </Badge>
            )}
            
            <CardHeader className="text-center">
              <div className="flex justify-center mb-2">
                {getPlanIcon(plan.tier)}
              </div>
              <CardTitle className="text-xl">{plan.name}</CardTitle>
              <CardDescription>{plan.description}</CardDescription>
              <div className="text-3xl font-bold">
                ${plan.price_monthly}
                <span className="text-sm font-normal text-muted-foreground">/month</span>
              </div>
              {plan.price_yearly && (
                <p className="text-sm text-muted-foreground">
                  or ${plan.price_yearly}/year (save 17%)
                </p>
              )}
            </CardHeader>

            <CardContent className="space-y-4">
              <ul className="space-y-2">
                {getFeatureList(plan.features).map((feature, index) => (
                  <li key={index} className="flex items-center gap-2 text-sm">
                    <Check className="h-4 w-4 text-green-500 flex-shrink-0" />
                    {feature}
                  </li>
                ))}
              </ul>

              <Button
                className="w-full"
                variant={isCurrentPlan(plan.id) ? 'secondary' : 'default'}
                disabled={isCurrentPlan(plan.id) || subscribing === plan.id}
                onClick={() => handleSubscribe(plan.id)}
              >
                {subscribing === plan.id ? (
                  'Processing...'
                ) : isCurrentPlan(plan.id) ? (
                  'Current Plan'
                ) : plan.tier === 'free' ? (
                  'Downgrade'
                ) : (
                  'Upgrade'
                )}
              </Button>
            </CardContent>
          </Card>
        ))}
      </div>

      {currentSubscription && (
        <Card>
          <CardHeader>
            <CardTitle>Current Subscription</CardTitle>
            <CardDescription>
              You are currently on the {currentSubscription.plan.name}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">
                  Next billing: {new Date(currentSubscription.subscription.current_period_end).toLocaleDateString()}
                </p>
                <p className="text-sm text-muted-foreground">
                  Status: {currentSubscription.subscription.status}
                </p>
              </div>
              {currentSubscription.subscription.status === 'active' && (
                <Button variant="outline" size="sm">
                  Manage Subscription
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}