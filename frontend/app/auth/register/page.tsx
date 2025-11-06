"use client"
import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { RegisterForm } from '@/components/auth/RegisterForm'
import { isLoginEnabled } from '@/lib/feature-toggles'

export default function RegisterPage() {
  const router = useRouter()

  useEffect(() => {
    if (!isLoginEnabled()) {
      router.replace('/dashboard')
    }
  }, [router])

  return <RegisterForm />
}