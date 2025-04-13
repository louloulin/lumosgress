import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import DeveloperPortalPage from '../app/dashboard/developer-portal/page'

describe('Developer Portal', () => {
  it('should render the developer portal title', () => {
    renderWithProviders(<DeveloperPortalPage />)
    expect(screen.getByRole('heading', { name: 'Developer Portal' })).toBeInTheDocument()
  })

  it('should render the portal description', () => {
    renderWithProviders(<DeveloperPortalPage />)
    expect(screen.getByText('Resources and documentation for integrating with the Proksi AI Gateway.')).toBeInTheDocument()
  })

  it('should render the tabs navigation', () => {
    renderWithProviders(<DeveloperPortalPage />)
    expect(screen.getByRole('tab', { name: 'Getting Started' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Documentation' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'SDK & Libraries' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'API Reference' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Examples' })).toBeInTheDocument()
  })

  it('should show getting started content by default', () => {
    renderWithProviders(<DeveloperPortalPage />)
    expect(screen.getByText('Welcome to Proksi AI Gateway')).toBeInTheDocument()
    expect(screen.getByText('Everything you need to get started with integrating our AI Gateway')).toBeInTheDocument()
  })

  it('should show core concepts section', () => {
    renderWithProviders(<DeveloperPortalPage />)
    expect(screen.getByText('Core Concepts')).toBeInTheDocument()
    expect(screen.getByText('LLM Routing')).toBeInTheDocument()
    expect(screen.getByText('Prompt Transformation')).toBeInTheDocument()
    expect(screen.getByText('Vector Operations')).toBeInTheDocument()
    expect(screen.getByText('AI Security')).toBeInTheDocument()
  })
}) 