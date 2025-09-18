'use client'

import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { useRouter } from 'next/navigation'
import { isLoginEnabled } from '@/lib/feature-toggles'

export interface User {
  id: string
  email: string
  name: string
  avatar_url?: string
  organization?: string
  role: 'admin' | 'user' | 'moderator'
  subscription_tier: 'free' | 'personal' | 'team' | 'enterprise'
  is_verified: boolean
  created_at: string
  last_login_at?: string
}

export interface AuthContextType {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  login: (email: string, password: string) => Promise<void>
  register: (data: RegisterData) => Promise<void>
  logout: () => void
  updateProfile: (data: Partial<User>) => Promise<void>
  changePassword: (currentPassword: string, newPassword: string) => Promise<void>
  token: string | null
}

export interface RegisterData {
  email: string
  password: string
  name: string
  avatar_url?: string
  organization?: string
}

interface AuthResponse {
  user: User
  token: string
  expires_at: string
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()
  const loginEnabled = isLoginEnabled()

  // Mock user for when login is disabled
  const mockUser: User = {
    id: 'dev-user',
    name: 'Development User',
    email: 'dev@conhub.local',
    avatar_url: undefined,
    organization: 'ConHub Dev',
    role: 'admin',
    subscription_tier: 'enterprise',
    is_verified: true,
    created_at: new Date().toISOString(),
    last_login_at: new Date().toISOString(),
  }

  useEffect(() => {
    if (!loginEnabled) {
      // When login is disabled, set mock user and finish loading
      setUser(mockUser)
      setToken('mock-token')
      setIsLoading(false)
      return
    }

    // Check for stored token on mount (only when login is enabled)
    const storedToken = localStorage.getItem('auth_token')
    if (storedToken) {
      setToken(storedToken)
      verifyToken(storedToken)
    } else {
      setIsLoading(false)
    }
  }, [loginEnabled])

  const verifyToken = async (tokenToVerify: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/verify`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${tokenToVerify}`,
          'Content-Type': 'application/json',
        },
      })

      if (response.ok) {
        const data = await response.json()
        if (data.valid) {
          // Get user profile
          await fetchUserProfile(tokenToVerify)
        } else {
          // Token is invalid, clear it
          localStorage.removeItem('auth_token')
          setToken(null)
          setUser(null)
        }
      } else {
        // Token verification failed
        localStorage.removeItem('auth_token')
        setToken(null)
        setUser(null)
      }
    } catch (error) {
      console.error('Token verification failed:', error)
      localStorage.removeItem('auth_token')
      setToken(null)
      setUser(null)
    } finally {
      setIsLoading(false)
    }
  }

  const fetchUserProfile = async (authToken: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/profile`, {
        headers: {
          'Authorization': `Bearer ${authToken}`,
          'Content-Type': 'application/json',
        },
      })

      if (response.ok) {
        const userData = await response.json()
        setUser(userData)
      } else {
        throw new Error('Failed to fetch user profile')
      }
    } catch (error) {
      console.error('Failed to fetch user profile:', error)
      localStorage.removeItem('auth_token')
      setToken(null)
      setUser(null)
    }
  }

  const login = async (email: string, password: string) => {
    if (!loginEnabled) {
      // When login is disabled, just redirect to dashboard
      router.push('/dashboard')
      return
    }

    setIsLoading(true)
    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      })

      if (response.ok) {
        const data: AuthResponse = await response.json()
        setUser(data.user)
        setToken(data.token)
        localStorage.setItem('auth_token', data.token)
        router.push('/dashboard')
      } else {
        const error = await response.json()
        throw new Error(error.error || 'Login failed')
      }
    } catch (error) {
      console.error('Login error:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const register = async (data: RegisterData) => {
    if (!loginEnabled) {
      // When login is disabled, just redirect to dashboard
      router.push('/dashboard')
      return
    }

    setIsLoading(true)
    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      })

      if (response.ok) {
        const authData: AuthResponse = await response.json()
        setUser(authData.user)
        setToken(authData.token)
        localStorage.setItem('auth_token', authData.token)
        router.push('/dashboard')
      } else {
        const error = await response.json()
        throw new Error(error.error || 'Registration failed')
      }
    } catch (error) {
      console.error('Registration error:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const logout = () => {
    if (!loginEnabled) {
      // When login is disabled, just redirect to homepage
      router.push('/')
      return
    }

    setUser(null)
    setToken(null)
    localStorage.removeItem('auth_token')
    router.push('/')
  }

  const updateProfile = async (data: Partial<User>) => {
    if (!loginEnabled) {
      // When login is disabled, update the mock user
      setUser(prev => prev ? { ...prev, ...data } : null)
      return
    }

    if (!token) throw new Error('No authentication token')

    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/profile`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      })

      if (response.ok) {
        const updatedUser = await response.json()
        setUser(updatedUser)
      } else {
        const error = await response.json()
        throw new Error(error.error || 'Profile update failed')
      }
    } catch (error) {
      console.error('Profile update error:', error)
      throw error
    }
  }

  const changePassword = async (currentPassword: string, newPassword: string) => {
    if (!loginEnabled) {
      // When login is disabled, just return success (no-op)
      return
    }

    if (!token) throw new Error('No authentication token')

    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/change-password`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword,
        }),
      })

      if (!response.ok) {
        const error = await response.json()
        throw new Error(error.error || 'Password change failed')
      }
    } catch (error) {
      console.error('Password change error:', error)
      throw error
    }
  }

  const value: AuthContextType = {
    user,
    isAuthenticated: !!user,
    isLoading,
    login,
    register,
    logout,
    updateProfile,
    changePassword,
    token,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}