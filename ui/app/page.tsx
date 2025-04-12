'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'

export default function Home() {
  const router = useRouter()
  
  useEffect(() => {
    router.push('/dashboard')
  }, [router])
  
  return (
    <main className="flex min-h-screen items-center justify-center">
      <div className="text-center">
        <h1 className="text-xl font-semibold">Redirecting to Dashboard...</h1>
        <p className="mt-2 text-muted-foreground">Please wait...</p>
      </div>
    </main>
  )
}
