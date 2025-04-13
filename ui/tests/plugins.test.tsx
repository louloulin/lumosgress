import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import PluginsPage from '../app/dashboard/plugins/page'

describe('Plugins Management', () => {
  it('should render the plugins page title', () => {
    renderWithProviders(<PluginsPage />)
    expect(screen.getByText('Plugins Management')).toBeInTheDocument()
  })

  it('should render the plugins description', () => {
    renderWithProviders(<PluginsPage />)
    expect(screen.getByText('Configure and manage your AI Gateway plugins')).toBeInTheDocument()
  })

  it('should render the install plugin button', () => {
    renderWithProviders(<PluginsPage />)
    expect(screen.getByText('Install Plugin')).toBeInTheDocument()
  })

  it('should render the plugins list', () => {
    renderWithProviders(<PluginsPage />)
    expect(screen.getByText('LLM Router')).toBeInTheDocument()
    expect(screen.getByText('Vector Database')).toBeInTheDocument()
    expect(screen.getByText('Prompt Debugger')).toBeInTheDocument()
    expect(screen.getByText('Anomaly Detection')).toBeInTheDocument()
  })

  it('should display plugin versions', () => {
    renderWithProviders(<PluginsPage />)
    const versionElements = screen.getAllByText('1.0.0')
    expect(versionElements.length).toBeGreaterThan(0)
  })
}) 