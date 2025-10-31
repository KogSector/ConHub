'use client'

import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { useRouter } from 'next/navigation'
import { apiClient } from '@/lib/api';
import { API_CONFIG } from '@/lib/config';

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

interface SessionData {
  token: string
  expires_at: string
  last_activity: string
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

// use API_CONFIG.baseUrl when a raw URL is required (e.g. for AbortController fetch)
// otherwise prefer apiClient for requests
const SESSION_TIMEOUT = 2 * 60 * 60 * 1000 

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()

  
  const saveSession = (token: string, expiresAt: string) => {
    const sessionData: SessionData = {
      token,
      expires_at: expiresAt,
      last_activity: new Date().toISOString()
    }
    localStorage.setItem('auth_session', JSON.stringify(sessionData))
    localStorage.setItem('auth_token', token) 
  }

  const getSession = (): SessionData | null => {
    try {
      const sessionStr = localStorage.getItem('auth_session')
      if (!sessionStr) return null
      return JSON.parse(sessionStr)
    } catch {
      return null
    }
  }

  const updateLastActivity = () => {
    const session = getSession()
    if (session) {
      session.last_activity = new Date().toISOString()
      localStorage.setItem('auth_session', JSON.stringify(session))
    }
  }

  const isSessionValid = (session: SessionData): boolean => {
    const now = new Date().getTime()
    const lastActivity = new Date(session.last_activity).getTime()
    const expiresAt = new Date(session.expires_at).getTime()
    
    return now < expiresAt && (now - lastActivity) < SESSION_TIMEOUT
  }

  const clearSession = () => {
    localStorage.removeItem('auth_session')
    localStorage.removeItem('auth_token')
    setToken(null)
    setUser(null)
  }

  useEffect(() => {
    
    const session = getSession()
    if (session && isSessionValid(session)) {
      setToken(session.token)
      updateLastActivity()
      
      const timeoutId = setTimeout(() => {
        setIsLoading(false)
      }, 3000) 
      
      verifyToken(session.token).finally(() => {
        clearTimeout(timeoutId)
      })
    } else {
      clearSession()
      setIsLoading(false)
    }
  }, [])

  
  useEffect(() => {
    if (token) {
      const handleActivity = () => updateLastActivity()
      
      window.addEventListener('mousedown', handleActivity)
      window.addEventListener('keydown', handleActivity)
      window.addEventListener('scroll', handleActivity)
      
      return () => {
        window.removeEventListener('mousedown', handleActivity)
        window.removeEventListener('keydown', handleActivity)
        window.removeEventListener('scroll', handleActivity)
      }
    }
  }, [token])

  const verifyToken = async (tokenToVerify: string) => {
    try {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 2000)

      const response = await fetch(`${API_CONFIG.baseUrl}/api/auth/verify`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${tokenToVerify}`,
          'Content-Type': 'application/json',
        },
        signal: controller.signal,
      })

      clearTimeout(timeoutId)

      if (response.ok) {
        const data = await response.json()
        if (data.valid) {
          await fetchUserProfile(tokenToVerify)
        } else {
          clearSession()
        }
      } else {
        clearSession()
      }
    } catch (error) {
      console.error('Token verification failed:', error)
      
      
      if (error.name === 'AbortError') {
        console.log('Token verification timed out, keeping session for offline use')
        
        setUser(null)
      } else {
        clearSession()
      }
    } finally {
      setIsLoading(false)
    }
  }

  const fetchUserProfile = async (authToken: string) => {
    try {
      const result = await apiClient.get('/api/auth/profile', { Authorization: `Bearer ${authToken}` }) as any;
      if (result?.success) {
        setUser(result.data)
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
    setIsLoading(true)
    try {
      const result = await apiClient.post('/api/auth/login', { email, password }) as any;

      if (result?.success) {
        const data: AuthResponse = result.data
        setUser(data.user)
        setToken(data.token)
        saveSession(data.token, data.expires_at)
        router.push('/dashboard')
      } else {
        throw new Error(result?.error || 'Login failed')
      }
    } catch (error) {
      console.error('Login error:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const register = async (data: RegisterData) => {
    setIsLoading(true)
    try {
      const result = await apiClient.post('/api/auth/register', data) as any;

      if (result?.success) {
        const authData: AuthResponse = result.data
        setUser(authData.user)
        setToken(authData.token)
        saveSession(authData.token, authData.expires_at)
        router.push('/dashboard')
      } else {
        throw new Error(result?.error || 'Registration failed')
      }
    } catch (error) {
      console.error('Registration error:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const logout = () => {
    clearSession()
    router.push('/')
  }

  const updateProfile = async (data: Partial<User>) => {
    if (!token) throw new Error('No authentication token')

    try {
      const result = await apiClient.put('/api/auth/profile', data, { Authorization: `Bearer ${token}` }) as any;

      if (result?.success) {
        setUser(result.data)
      } else {
        throw new Error(result?.error || 'Profile update failed')
      }
    } catch (error) {
      console.error('Profile update error:', error)
      throw error
    }
  }

  const changePassword = async (currentPassword: string, newPassword: string) => {
    if (!token) throw new Error('No authentication token')

    try {
      const result = await apiClient.post('/api/auth/change-password', {
        current_password: currentPassword,
        new_password: newPassword,
      }, { Authorization: `Bearer ${token}` }) as any;

      if (!result?.success) {
        throw new Error(result?.error || 'Password change failed')
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