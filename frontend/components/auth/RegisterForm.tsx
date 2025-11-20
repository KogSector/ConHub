'use client'

import { useState, useMemo } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useAuth } from '@/contexts/auth-context'
import { Eye, EyeOff, Mail, Lock, User, Building, ArrowRight, Sparkles } from 'lucide-react'
import Link from 'next/link'
import { signIn } from 'next-auth/react'
import { SocialLoginButtons } from './social-login-buttons'


const calculatePasswordStrength = (password: string): { score: number; label: string; color: string } => {
  let score = 0
  
  
  if (password.length >= 8) score += 20
  if (password.length >= 12) score += 10
  
  
  if (/[a-z]/.test(password)) score += 20
  if (/[A-Z]/.test(password)) score += 20
  if (/[0-9]/.test(password)) score += 15
  if (/[^A-Za-z0-9]/.test(password)) score += 15
  
  
  if (score < 30) return { score, label: 'Very Weak', color: 'bg-red-500' }
  if (score < 50) return { score, label: 'Weak', color: 'bg-orange-500' }
  if (score < 70) return { score, label: 'Fair', color: 'bg-yellow-500' }
  if (score < 90) return { score, label: 'Good', color: 'bg-blue-500' }
  return { score: 100, label: 'Strong', color: 'bg-green-500' }
}


const PasswordStrengthIndicator = ({ password }: { password: string }) => {
  const strength = useMemo(() => calculatePasswordStrength(password), [password])
  const filledDots = Math.ceil((strength.score / 100) * 10) 
  
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-400">Password Strength</span>
        {password && (
          <span className={`text-xs font-medium ${
            strength.score < 30 ? 'text-red-400' :
            strength.score < 50 ? 'text-orange-400' :
            strength.score < 70 ? 'text-yellow-400' :
            strength.score < 90 ? 'text-blue-400' : 'text-green-400'
          }`}>
            {strength.label}
          </span>
        )}
      </div>
      <div className="flex space-x-1">
        {Array.from({ length: 10 }, (_, i) => (
          <div
            key={i}
            className={`h-1.5 w-6 rounded-full transition-all duration-300 ${
              i < filledDots && password
                ? strength.color
                : 'bg-white/10'
            }`}
          />
        ))}
      </div>
    </div>
  )
}

export function RegisterForm() {
  const [formData, setFormData] = useState({
    email: '',
    password: '',
    confirmPassword: '',
    name: '',
    organization: '',
    avatar_url: ''
  })
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [socialLoading, setSocialLoading] = useState<string | null>(null)

  const { register } = useAuth()

  const handleSocialLogin = async (provider: string) => {
    setSocialLoading(provider)
    setError('')
    try {
      await signIn(provider, { callbackUrl: '/dashboard' })
    } catch (err) {
      setError(`Failed to sign in with ${provider}`)
      setSocialLoading(null)
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData(prev => ({
      ...prev,
      [e.target.name]: e.target.value
    }))
  }

  const validateForm = () => {
    if (formData.password !== formData.confirmPassword) {
      setError('Passwords do not match')
      return false
    }
    if (formData.password.length < 8) {
      setError('Password must be at least 8 characters long')
      return false
    }
    if (formData.name.length < 2) {
      setError('Name must be at least 2 characters long')
      return false
    }
    return true
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (!validateForm()) return

    setIsLoading(true)

    try {
      await register({
        email: formData.email,
        password: formData.password,
        name: formData.name,
        organization: formData.organization || undefined,
        avatar_url: formData.avatar_url || undefined
      })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Registration failed'
      
      
      if (errorMessage.toLowerCase().includes('already exists')) {
        setError('An account with this email already exists. Please sign in instead.')
      } else {
        setError(errorMessage)
      }
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 relative overflow-hidden py-8">
      {}
      <div className="absolute inset-0 bg-grid-pattern opacity-5"></div>
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-blue-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob"></div>
      <div className="absolute top-0 right-1/4 w-96 h-96 bg-purple-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-2000"></div>
      <div className="absolute bottom-8 left-1/3 w-96 h-96 bg-cyan-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-4000"></div>
      
      <div className="relative z-10 w-full max-w-md mx-4">
        {}
        <div className="text-center mb-8">
          <Link href="/" className="inline-flex items-center space-x-2 group">
            <Sparkles className="w-8 h-8 text-blue-400 group-hover:text-blue-300 transition-colors" />
            <span className="text-4xl font-bold font-orbitron bg-gradient-to-r from-blue-400 via-cyan-400 to-purple-400 bg-clip-text text-transparent">
              ConHub
            </span>
          </Link>
        </div>

        <Card className="backdrop-blur-xl bg-white/5 border border-white/10 shadow-2xl">
          <CardHeader className="space-y-1 text-center pb-6">
            <CardTitle className="text-3xl font-bold text-white">Join ConHub</CardTitle>
            <CardDescription className="text-gray-300">
              Create your account and start your journey
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            <form onSubmit={handleSubmit} className="space-y-5">
              {error && (
                <div className="p-4 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-lg backdrop-blur-sm">
                  {error}
                  {error.includes('An account with this email already exists') && (
                    <div className="mt-2">
                      <Link 
                        href="/auth/login" 
                        className="font-semibold text-blue-400 hover:text-blue-300 transition-colors hover:underline"
                      >
                        → Sign in instead
                      </Link>
                    </div>
                  )}
                </div>
              )}
              
              <div className="grid grid-cols-1 gap-5">
                <div className="space-y-2">
                  <Label htmlFor="name" className="text-gray-200 font-medium">Full Name</Label>
                  <div className="relative group">
                    <User className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-blue-400 transition-colors" />
                    <Input
                      id="name"
                      name="name"
                      type="text"
                      placeholder="Enter your full name"
                      value={formData.name}
                      onChange={handleChange}
                      className="pl-11 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-blue-400 focus:ring-blue-400/20 h-12"
                      required
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="email" className="text-gray-200 font-medium">Email Address</Label>
                  <div className="relative group">
                    <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-blue-400 transition-colors" />
                    <Input
                      id="email"
                      name="email"
                      type="email"
                      placeholder="Enter your email"
                      value={formData.email}
                      onChange={handleChange}
                      className="pl-11 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-blue-400 focus:ring-blue-400/20 h-12"
                      required
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="organization" className="text-gray-200 font-medium">Organization <span className="text-gray-400 text-sm">(Optional)</span></Label>
                  <div className="relative group">
                    <Building className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-blue-400 transition-colors" />
                    <Input
                      id="organization"
                      name="organization"
                      type="text"
                      placeholder="Your company or organization"
                      value={formData.organization}
                      onChange={handleChange}
                      className="pl-11 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-blue-400 focus:ring-blue-400/20 h-12"
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="password" className="text-gray-200 font-medium">Password</Label>
                  <div className="relative group">
                    <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-blue-400 transition-colors" />
                    <Input
                      id="password"
                      name="password"
                      type={showPassword ? 'text' : 'password'}
                      placeholder="Create a password (min. 8 characters)"
                      value={formData.password}
                      onChange={handleChange}
                      className="pl-11 pr-12 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-blue-400 focus:ring-blue-400/20 h-12"
                      required
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="absolute right-0 top-0 h-full px-3 text-gray-400 hover:text-blue-400 hover:bg-transparent"
                      onClick={() => setShowPassword(!showPassword)}
                    >
                      {showPassword ? (
                        <EyeOff className="h-5 w-5" />
                      ) : (
                        <Eye className="h-5 w-5" />
                      )}
                    </Button>
                  </div>
                  <PasswordStrengthIndicator password={formData.password} />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="confirmPassword" className="text-gray-200 font-medium">Confirm Password</Label>
                  <div className="relative group">
                    <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-blue-400 transition-colors" />
                    <Input
                      id="confirmPassword"
                      name="confirmPassword"
                      type={showConfirmPassword ? 'text' : 'password'}
                      placeholder="Confirm your password"
                      value={formData.confirmPassword}
                      onChange={handleChange}
                      className="pl-11 pr-12 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-blue-400 focus:ring-blue-400/20 h-12"
                      required
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="absolute right-0 top-0 h-full px-3 text-gray-400 hover:text-blue-400 hover:bg-transparent"
                      onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    >
                      {showConfirmPassword ? (
                        <EyeOff className="h-5 w-5" />
                      ) : (
                        <Eye className="h-5 w-5" />
                      )}
                    </Button>
                  </div>
                </div>
              </div>

              <Button
                type="submit"
                className="w-full h-12 bg-gradient-to-r from-blue-500 to-cyan-500 hover:from-blue-600 hover:to-cyan-600 text-white font-semibold rounded-lg transition-all duration-300 transform hover:scale-[1.02] shadow-lg hover:shadow-blue-500/25 group mt-8"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="flex items-center space-x-2">
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    <span>Creating account...</span>
                  </div>
                ) : (
                  <div className="flex items-center justify-center space-x-2">
                    <span>Create account</span>
                    <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
                  </div>
                )}
              </Button>

              <div className="text-center">
                <p className="text-gray-300">
                  Already have an account?{' '}
                  <Link 
                    href="/auth/login" 
                    className="font-semibold text-blue-400 hover:text-blue-300 transition-colors hover:underline"
                  >
                    Sign in
                  </Link>
                </p>
              </div>
            </form>

            {/* Social Login Section */}
            <div className="relative my-6">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-white/10"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-4 bg-slate-900/50 text-gray-400">Or continue with</span>
              </div>
            </div>
            <SocialLoginButtons 
              mode="signup" 
              onSocialLogin={handleSocialLogin}
              disabled={isLoading}
            />
          </CardContent>
        </Card>

        <div className="text-center mt-8">
          <p className="text-gray-400 text-sm">
            Join thousands of developers building the future
          </p>
        </div>
      </div>
    </div>
  )
}