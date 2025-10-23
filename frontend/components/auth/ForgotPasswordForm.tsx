'use client'

import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Mail, ArrowRight, Sparkles, ArrowLeft, CheckCircle } from 'lucide-react'
import Link from 'next/link'

export function ForgotPasswordForm() {
  const [email, setEmail] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitted, setIsSubmitted] = useState(false)
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setIsLoading(true)

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001'}/api/auth/forgot-password`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email }),
      })

      const data = await response.json()
      
      if (response.ok) {
        setIsSubmitted(true)
      } else {
        
        setIsSubmitted(true)
      }
      
    } catch (err) {
      
      setIsSubmitted(true)
    } finally {
      setIsLoading(false)
    }
  }

  if (isSubmitted) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 relative overflow-hidden">
        {}
        <div className="absolute inset-0 bg-grid-pattern opacity-5"></div>
        <div className="absolute top-0 left-1/4 w-96 h-96 bg-green-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob"></div>
        <div className="absolute top-0 right-1/4 w-96 h-96 bg-blue-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-2000"></div>
        <div className="absolute bottom-8 left-1/3 w-96 h-96 bg-purple-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-4000"></div>
        
        <div className="relative z-10 w-full max-w-md mx-4">
          {}
          <div className="text-center mb-8">
            <Link href="/" className="inline-flex items-center space-x-2 group">
              <Sparkles className="w-8 h-8 text-purple-400 group-hover:text-purple-300 transition-colors" />
              <span className="text-4xl font-bold font-orbitron bg-gradient-to-r from-purple-400 via-pink-400 to-blue-400 bg-clip-text text-transparent">
                ConHub
              </span>
            </Link>
          </div>

          <Card className="backdrop-blur-xl bg-white/5 border border-white/10 shadow-2xl">
            <CardHeader className="space-y-1 text-center pb-6">
              <div className="flex justify-center mb-4">
                <CheckCircle className="w-16 h-16 text-green-400" />
              </div>
              <CardTitle className="text-3xl font-bold text-white">Check your email</CardTitle>
              <CardDescription className="text-gray-300">
                We've sent a password reset link to
              </CardDescription>
              <p className="text-purple-400 font-medium">{email}</p>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="text-center space-y-4">
                <p className="text-gray-300 text-sm">
                  Didn't receive the email? Check your spam folder or try again.
                </p>
                
                <Button
                  onClick={() => setIsSubmitted(false)}
                  variant="outline"
                  className="w-full h-12 border-white/10 text-white hover:bg-white/10"
                >
                  Try different email
                </Button>

                <Link href="/auth/login">
                  <Button
                    variant="ghost"
                    className="w-full h-12 text-purple-400 hover:text-purple-300 hover:bg-transparent"
                  >
                    <ArrowLeft className="w-5 h-5 mr-2" />
                    Back to sign in
                  </Button>
                </Link>
              </div>
            </CardContent>
          </Card>

          <div className="text-center mt-8">
            <p className="text-gray-400 text-sm">
              Password reset email powered by ConHub
            </p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 relative overflow-hidden">
      {}
      <div className="absolute inset-0 bg-grid-pattern opacity-5"></div>
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-purple-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob"></div>
      <div className="absolute top-0 right-1/4 w-96 h-96 bg-blue-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-2000"></div>
      <div className="absolute bottom-8 left-1/3 w-96 h-96 bg-pink-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-4000"></div>
      
      <div className="relative z-10 w-full max-w-md mx-4">
        {}
        <div className="text-center mb-8">
          <Link href="/" className="inline-flex items-center space-x-2 group">
            <Sparkles className="w-8 h-8 text-purple-400 group-hover:text-purple-300 transition-colors" />
            <span className="text-4xl font-bold font-orbitron bg-gradient-to-r from-purple-400 via-pink-400 to-blue-400 bg-clip-text text-transparent">
              ConHub
            </span>
          </Link>
        </div>

        <Card className="backdrop-blur-xl bg-white/5 border border-white/10 shadow-2xl">
          <CardHeader className="space-y-1 text-center pb-6">
            <CardTitle className="text-3xl font-bold text-white">Reset your password</CardTitle>
            <CardDescription className="text-gray-300">
              Enter your email address and we'll send you a link to reset your password
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <form onSubmit={handleSubmit} className="space-y-6">
              {error && (
                <div className="p-4 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-lg backdrop-blur-sm">
                  {error}
                </div>
              )}
              
              <div className="space-y-2">
                <Label htmlFor="email" className="text-gray-200 font-medium">Email Address</Label>
                <div className="relative group">
                  <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-purple-400 transition-colors" />
                  <Input
                    id="email"
                    type="email"
                    placeholder="Enter your email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="pl-11 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-purple-400 focus:ring-purple-400/20 h-12"
                    required
                  />
                </div>
              </div>

              <Button
                type="submit"
                className="w-full h-12 bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white font-semibold rounded-lg transition-all duration-300 transform hover:scale-[1.02] shadow-lg hover:shadow-purple-500/25 group"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="flex items-center space-x-2">
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    <span>Sending reset email...</span>
                  </div>
                ) : (
                  <div className="flex items-center justify-center space-x-2">
                    <span>Send reset email</span>
                    <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
                  </div>
                )}
              </Button>

              <div className="text-center">
                <Link href="/auth/login">
                  <Button
                    variant="ghost"
                    className="text-purple-400 hover:text-purple-300 hover:bg-transparent"
                  >
                    <ArrowLeft className="w-4 h-4 mr-2" />
                    Back to sign in
                  </Button>
                </Link>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="text-center mt-8">
          <p className="text-gray-400 text-sm">
            Secure password reset powered by ConHub
          </p>
        </div>
      </div>
    </div>
  )
}