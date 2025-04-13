import { render, screen, fireEvent } from '@testing-library/react'
import { ThemeToggle } from '@/components/ui/theme-toggle'
import { ThemeProvider } from '@/lib/theme-provider'
import '@testing-library/jest-dom'
import { vi, describe, it, expect } from 'vitest'

// Mock next-themes
vi.mock('next-themes', () => ({
  useTheme: () => ({
    theme: 'light',
    setTheme: vi.fn(),
  }),
}))

describe('ThemeToggle', () => {
  it('renders theme toggle button', () => {
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    )
    
    // The toggle button should be in the document
    const button = screen.getByRole('button')
    expect(button).toBeInTheDocument()
    
    // Should have the toggle theme text as accessible name
    expect(screen.getByText('Toggle theme')).toBeInTheDocument()
  })
  
  it('opens dropdown menu when clicked', () => {
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    )
    
    // Click the toggle button
    const button = screen.getByRole('button')
    fireEvent.click(button)
    
    // The dropdown menu should appear with theme options
    expect(screen.getByText('Light')).toBeInTheDocument()
    expect(screen.getByText('Dark')).toBeInTheDocument()
    expect(screen.getByText('System')).toBeInTheDocument()
  })
}) 