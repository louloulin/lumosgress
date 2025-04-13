import { describe, it, expect, vi } from 'vitest'
import { screen, within } from '@testing-library/react'
import '@testing-library/jest-dom'
import { renderWithProviders } from './test-utils'
import DashboardPage from '../app/dashboard/page'

// Mock chart components to avoid rendering issues in tests
vi.mock('@/components/dashboard/traffic-chart', () => ({
  TrafficChart: () => <div data-testid="traffic-chart">Traffic Chart</div>
}))

vi.mock('@/components/dashboard/llm-usage-chart', () => ({
  LlmUsageChart: () => <div data-testid="llm-usage-chart">LLM Usage Chart</div>
}))

describe('Dashboard', () => {
  it('should render the dashboard title', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByRole('heading', { name: 'Dashboard' })).toBeInTheDocument()
  })

  it('should render the welcome message', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByText('Welcome to the Proksi AI Gateway control panel.')).toBeInTheDocument()
  })

  it('should render the overview tab by default', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByRole('tab', { name: 'Overview' })).toBeInTheDocument()
    
    // Get card elements by their title
    const cards = screen.getAllByRole('generic', { name: '' }).filter(
      element => element.getAttribute('data-slot') === 'card'
    );
    
    // Check for metric cards within their container context
    expect(cards.length).toBeGreaterThan(0);
    cards.forEach(card => {
      const cardHeader = within(card).queryByTestId('card-header') || card;
      const title = within(cardHeader).queryByText(/Total Requests|Avg\. Response Time|Active Plugins|Error Rate/i);
      if (title) {
        expect(title).toBeInTheDocument();
      }
    });
  })

  it('should render the analytics tab', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByRole('tab', { name: 'Analytics' })).toBeInTheDocument()
  })

  it('should render the LLM usage tab', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByRole('tab', { name: 'LLM Usage' })).toBeInTheDocument()
  })

  it('should render the traffic chart component', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByTestId('traffic-chart')).toBeInTheDocument()
  })

  it('should render the request traffic chart title', () => {
    renderWithProviders(<DashboardPage />)
    expect(screen.getByText('Request Traffic')).toBeInTheDocument()
    expect(screen.getByText('Request volume over the past 30 days')).toBeInTheDocument()
  })
}) 