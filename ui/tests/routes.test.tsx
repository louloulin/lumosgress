import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import RoutesPage from '../app/dashboard/routes/page'

describe('Routes Management', () => {
  it('should render the routes page title', () => {
    renderWithProviders(<RoutesPage />)
    expect(screen.getByText('Routes Management')).toBeInTheDocument()
  })

  it('should render the routes description', () => {
    renderWithProviders(<RoutesPage />)
    expect(screen.getByText('Configure and manage your AI Gateway routes')).toBeInTheDocument()
  })

  it('should render the add new route button', () => {
    renderWithProviders(<RoutesPage />)
    expect(screen.getByText('Add New Route')).toBeInTheDocument()
  })

  it('should render the routes list', () => {
    renderWithProviders(<RoutesPage />)
    expect(screen.getByText('OpenAI Completions')).toBeInTheDocument()
    expect(screen.getByText('OpenAI Chat')).toBeInTheDocument()
    expect(screen.getByText('Anthropic Messages')).toBeInTheDocument()
    expect(screen.getByText('Vector Search')).toBeInTheDocument()
    expect(screen.getByText('Vector Upsert')).toBeInTheDocument()
  })

  it('should display route paths', () => {
    renderWithProviders(<RoutesPage />)
    expect(screen.getByText('/v1/completions')).toBeInTheDocument()
    expect(screen.getByText('/v1/chat/completions')).toBeInTheDocument()
    expect(screen.getByText('/v1/messages')).toBeInTheDocument()
    expect(screen.getByText('/vectors/search')).toBeInTheDocument()
    expect(screen.getByText('/vectors/upsert')).toBeInTheDocument()
  })
}) 