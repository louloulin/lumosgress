import { describe, it, expect } from 'vitest'
import { screen, fireEvent } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import DocumentationPage from '../app/dashboard/documentation/page'

describe('Documentation Page', () => {
  it('should render the page title', () => {
    renderWithProviders(<DocumentationPage />)
    expect(screen.getByRole('heading', { name: 'Documentation' })).toBeInTheDocument()
  })

  it('should render the page description', () => {
    renderWithProviders(<DocumentationPage />)
    expect(screen.getByText('Guides, tutorials, and API reference for Proksi AI Gateway')).toBeInTheDocument()
  })

  it('should render the search input', () => {
    renderWithProviders(<DocumentationPage />)
    expect(screen.getByPlaceholderText('Search documentation...')).toBeInTheDocument()
  })

  it('should render category filter buttons', () => {
    renderWithProviders(<DocumentationPage />)
    expect(screen.getByRole('button', { name: 'All' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Getting Started/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Core Features/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Advanced Usage/i })).toBeInTheDocument()
  })

  it('should render the content tabs', () => {
    renderWithProviders(<DocumentationPage />)
    expect(screen.getByRole('tab', { name: 'Guides & Reference' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Tutorials' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Video Guides' })).toBeInTheDocument()
  })

  it('should display documentation content by default', () => {
    renderWithProviders(<DocumentationPage />)
    expect(screen.getByText('Getting Started')).toBeInTheDocument()
    expect(screen.getByText('Introduction to Proksi')).toBeInTheDocument()
    expect(screen.getByText('Quick Start Guide')).toBeInTheDocument()
  })

  it('should switch to tutorials tab when clicked', () => {
    renderWithProviders(<DocumentationPage />)
    fireEvent.click(screen.getByRole('tab', { name: 'Tutorials' }))
    expect(screen.getByText('Building a Basic RAG Application')).toBeInTheDocument()
    expect(screen.getByText('Streaming Chat Responses')).toBeInTheDocument()
  })
}) 