'use client'

import { useState, useEffect, useCallback } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export const dynamic = 'force-dynamic'

export default function ApiTestPage() {
  const testApi = async () => {
    const response = await fetch('http://localhost:3001/health')
    if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`)
    return response.json()
  }
  const [status, setStatus] = useState<string>('Not tested')
  const [isLoading, setIsLoading] = useState(false)

  const testConnection = useCallback(async () => {
    setIsLoading(true)
    try {
      const response = await testApi()
      setStatus(`✅ Connected! Backend responded: ${response.service}`)
    } catch (error) {
      setStatus(`❌ Connection failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    // Test connection on page load
    testConnection()
  }, [testConnection])

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-2xl mx-auto">
        <Card>
          <CardHeader>
            <CardTitle>API Connection Test</CardTitle>
            <CardDescription>
              Testing connection between Next.js frontend and Rust Actix backend
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="p-4 bg-muted rounded-lg">
              <p className="font-mono text-sm">
                Frontend: http://localhost:3000
              </p>
              <p className="font-mono text-sm">
                Backend: http://localhost:3001
              </p>
            </div>
            
            <div className="space-y-2">
              <p className="font-semibold">Connection Status:</p>
              <p className="text-sm">{status}</p>
            </div>

            <Button 
              onClick={testConnection} 
              disabled={isLoading}
              className="w-full"
            >
              {isLoading ? 'Testing...' : 'Test Connection'}
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
