import React, { ReactNode } from 'react'
import { render } from '@testing-library/react'
import { vi } from 'vitest'

// Mock ResizeObserver
class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// Add ResizeObserver to the global object
global.ResizeObserver = ResizeObserverMock;

// Mock the auth context
vi.mock('@/lib/auth-provider', () => ({
  useAuth: () => ({
    user: { name: 'Test User', email: 'test@example.com', role: 'admin' },
    login: vi.fn(),
    logout: vi.fn(),
    isLoading: false,
  }),
  AuthProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}))

// Mock the DashboardLayout component
vi.mock('@/components/layout/dashboard-layout', () => ({
  DashboardLayout: ({ children }: { children: ReactNode }) => <>{children}</>,
}))

export function renderWithProviders(ui: React.ReactElement) {
  return render(ui)
} 