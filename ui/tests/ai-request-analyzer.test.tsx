import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import AIRequestAnalyzerPage from '../app/dashboard/tools/ai-request-analyzer/page'

describe('AI Request Analyzer', () => {
  it('should render the analyzer page title', () => {
    renderWithProviders(<AIRequestAnalyzerPage />)
    expect(screen.getByRole('heading', { name: 'AI Request Analyzer' })).toBeInTheDocument()
  })

  it('should render the analyzer description', () => {
    renderWithProviders(<AIRequestAnalyzerPage />)
    expect(screen.getByText('Analyze and optimize your AI API requests for better performance and cost efficiency')).toBeInTheDocument()
  })

  it('should render the request configuration section', () => {
    renderWithProviders(<AIRequestAnalyzerPage />)
    expect(screen.getByText('Request Configuration')).toBeInTheDocument()
    expect(screen.getByText('Enter your API request JSON to analyze for quality, efficiency and cost')).toBeInTheDocument()
  })

  it('should render the template buttons', () => {
    renderWithProviders(<AIRequestAnalyzerPage />)
    expect(screen.getByText('OpenAI Chat Completion')).toBeInTheDocument()
    expect(screen.getByText('OpenAI Function Call')).toBeInTheDocument()
    expect(screen.getByText('Anthropic Completion')).toBeInTheDocument()
  })

  it('should render the model selector', () => {
    renderWithProviders(<AIRequestAnalyzerPage />)
    expect(screen.getByText('Model')).toBeInTheDocument()
  })

  it('should render the analyze button', () => {
    renderWithProviders(<AIRequestAnalyzerPage />)
    expect(screen.getByRole('button', { name: 'Analyze Request' })).toBeInTheDocument()
  })
}) 