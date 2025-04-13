import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import AIFeaturesPage from '../app/dashboard/ai-features/page'

describe('AI Features', () => {
  it('should render the AI features page title', () => {
    renderWithProviders(<AIFeaturesPage />)
    expect(screen.getByText('AI Features')).toBeInTheDocument()
  })

  it('should render the AI features description', () => {
    renderWithProviders(<AIFeaturesPage />)
    expect(screen.getByText('Configure and manage AI gateway capabilities')).toBeInTheDocument()
  })

  it('should render the prompt templates section', () => {
    renderWithProviders(<AIFeaturesPage />)
    expect(screen.getByText('Prompt Templates')).toBeInTheDocument()
    expect(screen.getByText('Create and manage prompt templates for different use cases')).toBeInTheDocument()
  })

  it('should render the vector databases section', () => {
    renderWithProviders(<AIFeaturesPage />)
    expect(screen.getByText('Vector Databases')).toBeInTheDocument()
    expect(screen.getByText('Configure vector database integrations for RAG workflows')).toBeInTheDocument()
  })

  it('should render the LLM providers section', () => {
    renderWithProviders(<AIFeaturesPage />)
    expect(screen.getByText('LLM Providers')).toBeInTheDocument()
    expect(screen.getByText('Configure and manage LLM provider integrations')).toBeInTheDocument()
  })

  it('should render the safety & moderation section', () => {
    renderWithProviders(<AIFeaturesPage />)
    expect(screen.getByText('Safety & Moderation')).toBeInTheDocument()
    expect(screen.getByText('Configure content moderation and safety settings')).toBeInTheDocument()
  })
}) 