import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Authentication - Proksi AI Gateway',
  description: 'Authentication page for Proksi AI Gateway',
}

export default function AuthLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="min-h-screen bg-gradient-to-br from-background to-muted">
      <div className="container relative flex min-h-screen flex-col items-center justify-center">
        <div className="absolute top-8 left-8">
          <div className="flex items-center gap-2">
            <span className="font-bold text-2xl">Proksi</span>
            <span className="px-2 py-1 text-xs font-semibold rounded-md bg-primary text-primary-foreground">
              AI Gateway
            </span>
          </div>
        </div>
        {children}
      </div>
    </div>
  )
} 