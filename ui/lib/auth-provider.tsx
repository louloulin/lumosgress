'use client'

import { createContext, useContext, useEffect, useState } from 'react'
import { useRouter, usePathname } from 'next/navigation'

type User = {
  name: string
  role: string
}

interface AuthContextType {
  user: User | null
  isLoading: boolean
  login: (username: string, password: string) => Promise<void>
  logout: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

// Check if we're in development mode
const isDevelopment = process.env.NODE_ENV === 'development'

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()
  const pathname = usePathname()

  // Check if user is logged in on mount
  useEffect(() => {
    const checkAuth = () => {
      try {
        // In development mode, auto-login with a test user
        if (isDevelopment) {
          setUser({ name: 'dev_user', role: 'admin' })
        } else {
          const token = localStorage.getItem('auth_token')
          const storedUser = localStorage.getItem('user')
          
          if (token && storedUser) {
            setUser(JSON.parse(storedUser))
          }
        }
      } catch (error) {
        console.error('Authentication error:', error)
      } finally {
        setIsLoading(false)
      }
    }
    
    checkAuth()
  }, [])
  
  // Redirect unauthenticated users away from protected routes
  useEffect(() => {
    if (!isLoading) {
      // Skip auth redirects in development mode
      if (isDevelopment) return
      
      const isAuthPage = pathname?.startsWith('/auth')
      
      if (!user && !isAuthPage && pathname !== '/') {
        // Redirect to login if accessing protected page while not logged in
        router.push('/auth/login')
      } else if (user && isAuthPage) {
        // Redirect to dashboard if accessing auth page while logged in
        router.push('/dashboard')
      }
    }
  }, [user, isLoading, pathname, router])

  const login = async (username: string, password: string) => {
    setIsLoading(true)
    
    try {
      // In development mode, accept any credentials
      if (isDevelopment) {
        const devUser = { name: username || 'dev_user', role: 'admin' }
        setUser(devUser)
        router.push('/dashboard')
        return
      }
      
      // In a real app, this would call an API endpoint
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // For demo, accept any non-empty username/password
      if (!username || !password) {
        throw new Error('Username and password are required')
      }
      
      const newUser = { name: username, role: 'admin' }
      
      // Store in localStorage
      localStorage.setItem('auth_token', 'demo_token')
      localStorage.setItem('user', JSON.stringify(newUser))
      
      setUser(newUser)
      router.push('/dashboard')
    } catch (error) {
      console.error('Login error:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const logout = () => {
    setIsLoading(true)
    
    try {
      // Skip in development mode
      if (isDevelopment) {
        setUser(null)
        return
      }
      
      // Clear auth data
      localStorage.removeItem('auth_token')
      localStorage.removeItem('user')
      
      setUser(null)
      router.push('/auth/login')
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <AuthContext.Provider value={{ user, isLoading, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const context = useContext(AuthContext)
  
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  
  return context
} 