'use client'

import React, { createContext, useContext, useEffect, useState, ReactNode, useCallback, useMemo } from 'react'
import { useRouter } from 'next/navigation'
import { apiClient, ApiResponse, unwrapResponse } from '@/lib/api';
import { isLoginEnabled } from '@/lib/feature-toggles';
// Use a fixed dev user ID to avoid bundling issues with uuid in client

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
  const loginEnabled = isLoginEnabled()

  // Default dev user when auth is disabled
  const devUser: User = useMemo(() => ({
    id: 'dev-user-00000000',
    email: 'dev@conhub.local',
    name: 'Development User',
    avatar_url: undefined,
    organization: 'ConHub Dev',
    role: 'user',
    subscription_tier: 'free',
    is_verified: true,
    created_at: new Date().toISOString(),
    last_login_at: new Date().toISOString()
  }), [])

  
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

  const updateLastActivity = useCallback(() => {
    const session = getSession()
    if (session) {
      session.last_activity = new Date().toISOString()
      localStorage.setItem('auth_session', JSON.stringify(session))
    }
  }, [])

  const isSessionValid = (session: SessionData): boolean => {
    const now = new Date().getTime()
    const lastActivity = new Date(session.last_activity).getTime()
    const expiresAt = new Date(session.expires_at).getTime()
    
    return now < expiresAt && (now - lastActivity) < SESSION_TIMEOUT
  }

  const clearSession = useCallback(() => {
    localStorage.removeItem('auth_session')
    localStorage.removeItem('auth_token')
    setToken(null)
    // Preserve default dev user when auth is disabled
    setUser(loginEnabled ? null : devUser)
  }, [loginEnabled, devUser])

  // Fetch the user profile using the current token
  const fetchUserProfile = useCallback(async (authToken: string) => {
    try {
      const result = await apiClient.get<ApiResponse<User>>('/api/auth/profile', { Authorization: `Bearer ${authToken}` })
      if (result?.success && result.data) {
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
  }, [])

  // Verify the token with the backend and update session/user state accordingly
  const verifyToken = useCallback(async (tokenToVerify: string) => {
    // Skip verification entirely when login is disabled
    if (!loginEnabled) {
      setIsLoading(false)
      return
    }
    try {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 2000)

      const authBase = process.env.NEXT_PUBLIC_AUTH_SERVICE_URL || 'http://localhost:3010';
      const response = await fetch(`${authBase}/api/auth/verify`, {
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
    } catch (err: unknown) {
      console.error('Token verification failed:', err)
      const isAbort = err instanceof Error && err.name === 'AbortError'
      if (isAbort) {
        console.log('Token verification timed out, keeping session for offline use')
        setUser(null)
      } else {
        clearSession()
      }
    } finally {
      setIsLoading(false)
    }
  }, [loginEnabled, fetchUserProfile, clearSession])

  useEffect(() => {
    // If login is disabled, provide a default dev user immediately
    if (!loginEnabled) {
      setUser(devUser)
      setToken(null)
      setIsLoading(false)
      return
    }

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
  }, [loginEnabled, updateLastActivity, verifyToken, clearSession, devUser])

  
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
  }, [token, updateLastActivity])

  const login = async (email: string, password: string) => {
    setIsLoading(true)
    try {
      // Bypass login when auth is disabled
      if (!loginEnabled) {
        setUser(devUser)
        setToken(null)
        router.push('/dashboard')
        return
      }
      const result = await apiClient.post('/api/auth/login', { email, password })
      const data = unwrapResponse<AuthResponse>(result)

      if (data && data.user && data.token) {
        setUser(data.user)
        setToken(data.token)
        saveSession(data.token, data.expires_at)
        router.push('/dashboard')
      } else {
        throw new Error('Login failed')
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
      // Bypass registration when auth is disabled
      if (!loginEnabled) {
        setUser(devUser)
        setToken(null)
        router.push('/dashboard')
        return
      }
      const result = await apiClient.post('/api/auth/register', data)
      const authData = unwrapResponse<AuthResponse>(result)

      if (authData && authData.user && authData.token) {
        setUser(authData.user)
        setToken(authData.token)
        saveSession(authData.token, authData.expires_at)
        router.push('/dashboard')
      } else {
        throw new Error('Registration failed')
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
      const result = await apiClient.put<ApiResponse<User>>('/api/auth/profile', data, { Authorization: `Bearer ${token}` })

      if (result?.success && result.data) {
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
      const result = await apiClient.post<ApiResponse>('/api/auth/change-password', {
        current_password: currentPassword,
        new_password: newPassword,
      }, { Authorization: `Bearer ${token}` })

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
