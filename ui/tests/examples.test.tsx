import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import ExamplesPage from '../app/dashboard/examples/page'

describe('Examples and Templates Page', () => {
  it('should render the page title', () => {
    renderWithProviders(<ExamplesPage />)
    expect(screen.getByRole('heading', { name: 'Examples & Templates' })).toBeInTheDocument()
  })

  it('should render the page description', () => {
    renderWithProviders(<ExamplesPage />)
    expect(screen.getByText('Reference examples, starter templates and code snippets for integrating with Proksi AI Gateway')).toBeInTheDocument()
  })

  it('should render tabs for different content types', () => {
    renderWithProviders(<ExamplesPage />)
    expect(screen.getByRole('tab', { name: 'Example Projects' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Starter Templates' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Code Snippets' })).toBeInTheDocument()
  })

  it('should render example projects by default', () => {
    renderWithProviders(<ExamplesPage />)
    expect(screen.getByText('AI Chat Application')).toBeInTheDocument()
    expect(screen.getByText('RAG Knowledge Base')).toBeInTheDocument()
    expect(screen.getByText('Multi-Model Router')).toBeInTheDocument()
  })

  it('should display project categories', () => {
    renderWithProviders(<ExamplesPage />)
    expect(screen.queryAllByText('nextjs').length).toBeGreaterThan(0)
    expect(screen.queryAllByText('vector-db').length).toBeGreaterThan(0)
    expect(screen.queryAllByText('beginner').length).toBeGreaterThan(0)
    expect(screen.queryAllByText('intermediate').length).toBeGreaterThan(0)
  })

  it('should render action buttons for projects', () => {
    renderWithProviders(<ExamplesPage />)
    expect(screen.getAllByText('View Code').length).toBeGreaterThan(0)
    expect(screen.getAllByText('Live Demo').length).toBeGreaterThan(0)
  })
}) 