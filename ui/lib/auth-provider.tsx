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

// Default dev user
const DEFAULT_DEV_USER = { name: 'dev_user', role: 'admin' }

export function AuthProvider({ children }: { children: React.ReactNode }) {
  // In development, initialize with a default user
  const [user, setUser] = useState<User | null>(isDevelopment ? DEFAULT_DEV_USER : null)
  const [isLoading, setIsLoading] = useState(false) // Set to false initially in dev mode
  const router = useRouter()
  const pathname = usePathname()

  // Check if user is logged in on mount (only in production)
  useEffect(() => {
    if (isDevelopment) return; // Skip in development mode
    
    const checkAuth = () => {
      setIsLoading(true);
      try {
        const token = localStorage.getItem('auth_token')
        const storedUser = localStorage.getItem('user')
        
        if (token && storedUser) {
          setUser(JSON.parse(storedUser))
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
    // Skip auth redirects in development mode
    if (isDevelopment) return;
    
    if (!isLoading) {
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
    // Skip in development mode - just keep default user
    if (isDevelopment) {
      setUser(DEFAULT_DEV_USER)
      return
    }
    
    setIsLoading(true)
    try {
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