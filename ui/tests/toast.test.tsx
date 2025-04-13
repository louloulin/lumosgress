import { render, screen, act } from '@testing-library/react'
import { useToast } from '@/components/ui/use-toast'
import { Toaster } from '@/components/ui/toaster'
import '@testing-library/jest-dom'
import { describe, it, expect } from 'vitest'

// Test component to trigger toast
function ToastTester() {
  const { toast, success, error, warning, info } = useToast()
  
  return (
    <div>
      <button onClick={() => toast({ title: 'Default Toast', description: 'This is a default toast' })}>
        Show Default Toast
      </button>
      <button onClick={() => success({ title: 'Success Toast', description: 'Operation completed successfully' })}>
        Show Success Toast
      </button>
      <button onClick={() => error({ title: 'Error Toast', description: 'Something went wrong' })}>
        Show Error Toast
      </button>
      <button onClick={() => warning({ title: 'Warning Toast', description: 'Be careful with this action' })}>
        Show Warning Toast
      </button>
      <button onClick={() => info({ title: 'Info Toast', description: 'Here is some information' })}>
        Show Info Toast
      </button>
    </div>
  )
}

describe('Toast System', () => {
  it('renders toast when triggered', () => {
    render(
      <>
        <ToastTester />
        <Toaster />
      </>
    )
    
    // Initially, no toasts should be visible
    expect(screen.queryByText('Default Toast')).not.toBeInTheDocument()
    
    // Trigger a default toast
    act(() => {
      screen.getByText('Show Default Toast').click()
    })
    
    // The toast should appear with title and description
    expect(screen.getByText('Default Toast')).toBeInTheDocument()
    expect(screen.getByText('This is a default toast')).toBeInTheDocument()
  })
  
  it('renders different toast variants', () => {
    render(
      <>
        <ToastTester />
        <Toaster />
      </>
    )
    
    // Trigger a success toast
    act(() => {
      screen.getByText('Show Success Toast').click()
    })
    
    // The success toast should appear
    expect(screen.getByText('Success Toast')).toBeInTheDocument()
    expect(screen.getByText('Operation completed successfully')).toBeInTheDocument()
    
    // Trigger an error toast
    act(() => {
      screen.getByText('Show Error Toast').click()
    })
    
    // The error toast should appear
    expect(screen.getByText('Error Toast')).toBeInTheDocument()
    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
    
    // Trigger a warning toast
    act(() => {
      screen.getByText('Show Warning Toast').click()
    })
    
    // The warning toast should appear
    expect(screen.getByText('Warning Toast')).toBeInTheDocument()
    expect(screen.getByText('Be careful with this action')).toBeInTheDocument()
    
    // Trigger an info toast
    act(() => {
      screen.getByText('Show Info Toast').click()
    })
    
    // The info toast should appear
    expect(screen.getByText('Info Toast')).toBeInTheDocument()
    expect(screen.getByText('Here is some information')).toBeInTheDocument()
  })
}) 