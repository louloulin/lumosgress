import { describe, it, expect } from 'vitest'
import { NextRequest } from 'next/server'
import { POST } from '../app/api/prompt-debugger/route'

// Import types from our API route
interface RuleResult {
  passed: boolean
  message: string
  severity: 'error' | 'warning' | 'success'
}

interface AnalysisRule {
  id: string
  name: string
  description: string
  result: RuleResult
}

interface AnalysisResponse {
  score: number
  results: AnalysisRule[]
  improvedPrompt: string
}

describe('Prompt Debugger API', () => {
  it('should analyze a valid prompt', async () => {
    const request = new NextRequest('http://localhost:3000/api/prompt-debugger', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        prompt: 'Explain quantum computing in simple terms with examples. Format the response into sections for beginners.',
      }),
    })

    const response = await POST(request)
    const data = await response.json() as AnalysisResponse

    expect(response.status).toBe(200)
    expect(data).toHaveProperty('score')
    expect(data).toHaveProperty('results')
    expect(data).toHaveProperty('improvedPrompt')
    expect(Array.isArray(data.results)).toBe(true)
    expect(data.results.length).toBeGreaterThan(0)
  })

  it('should return an error for an invalid request', async () => {
    const request = new NextRequest('http://localhost:3000/api/prompt-debugger', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({}), // Missing prompt
    })

    const response = await POST(request)
    const data = await response.json() as { error: string }

    expect(response.status).toBe(400)
    expect(data).toHaveProperty('error')
  })

  it('should identify issues in a poor prompt', async () => {
    const request = new NextRequest('http://localhost:3000/api/prompt-debugger', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        prompt: 'Tell me about [topic]',
      }),
    })

    const response = await POST(request)
    const data = await response.json() as AnalysisResponse

    expect(response.status).toBe(200)
    expect(data.score).toBeLessThan(100)
    expect(data.results.some((r: AnalysisRule) => !r.result.passed)).toBe(true)
    expect(data.improvedPrompt).not.toBe('Tell me about [topic]')
  })

  it('should give a high score to a well-crafted prompt', async () => {
    const request = new NextRequest('http://localhost:3000/api/prompt-debugger', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        prompt: `
          Explain the concept of neural networks in machine learning.
          
          Please structure your response with the following sections:
          1. Basic definition for beginners (100 words)
          2. How neural networks learn (150 words)
          3. Common applications with examples (200 words)
          4. Limitations and challenges (100 words)
          
          Use simple analogies where possible and avoid technical jargon.
          Include a brief comparison with how the human brain works.
        `,
      }),
    })

    const response = await POST(request)
    const data = await response.json() as AnalysisResponse

    expect(response.status).toBe(200)
    expect(data.score).toBeGreaterThanOrEqual(80)
    expect(data.results.filter((r: AnalysisRule) => r.result.passed).length).toBeGreaterThanOrEqual(5)
  })
}) 