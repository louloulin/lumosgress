import { NextResponse } from 'next/server'

// Define rule types
type RuleSeverity = 'error' | 'warning' | 'success'

interface RuleResult {
  passed: boolean
  message: string
  severity: RuleSeverity
}

interface PromptRule {
  id: string
  name: string
  description: string
  check: (prompt: string) => RuleResult
}

// Define the rules for prompt analysis
const promptRules: PromptRule[] = [
  {
    id: 'clarity',
    name: 'Clarity and Specificity',
    description: 'Checks if the prompt is clear and specific enough',
    check: (prompt: string) => {
      const wordCount = prompt.split(/\s+/).length
      if (wordCount < 5) {
        return { 
          passed: false, 
          message: 'Prompt is too short. Consider adding more specific instructions.', 
          severity: 'error' 
        }
      }
      if (prompt.includes('[') && prompt.includes(']')) {
        return {
          passed: false,
          message: 'Prompt contains placeholder text. Replace [placeholders] with actual content.',
          severity: 'error'
        }
      }
      if (wordCount < 15) {
        return { 
          passed: false, 
          message: 'Prompt could be more detailed. Add more context or specific requirements.', 
          severity: 'warning' 
        }
      }
      return { passed: true, message: 'Prompt is specific and clear.', severity: 'success' }
    }
  },
  {
    id: 'context',
    name: 'Context and Background',
    description: 'Evaluates if sufficient context is provided',
    check: (prompt: string) => {
      const hasContextMarkers = /background|context|previous|situation|scenario/i.test(prompt)
      if (!hasContextMarkers && prompt.length < 100) {
        return {
          passed: false,
          message: 'Consider adding more background context to help the AI understand the situation.',
          severity: 'warning'
        }
      }
      return { passed: true, message: 'Prompt contains good contextual information.', severity: 'success' }
    }
  },
  {
    id: 'structure',
    name: 'Structure and Format',
    description: 'Checks if the prompt has good structure',
    check: (prompt: string) => {
      const hasSections = /\n\n/.test(prompt) || /\d\.\s/.test(prompt) || /\*\s/.test(prompt)
      const hasParagraphs = prompt.split('\n').filter(p => p.trim().length > 0).length > 1
      
      if (!hasSections && !hasParagraphs && prompt.length > 150) {
        return {
          passed: false,
          message: 'Long prompt without clear structure. Consider breaking it into sections or bullet points.',
          severity: 'warning'
        }
      }
      return { passed: true, message: 'Prompt has good structure and organization.', severity: 'success' }
    }
  },
  {
    id: 'instructions',
    name: 'Clear Instructions',
    description: 'Evaluates if the instructions are clear',
    check: (prompt: string) => {
      const hasInstructionVerbs = /explain|describe|analyze|list|compare|summarize|provide|create|write|generate/i.test(prompt)
      if (!hasInstructionVerbs) {
        return {
          passed: false,
          message: 'No clear instruction verbs found. Add explicit instructions like "explain", "describe", or "analyze".',
          severity: 'error'
        }
      }
      return { passed: true, message: 'Prompt contains clear instruction verbs.', severity: 'success' }
    }
  },
  {
    id: 'constraints',
    name: 'Output Constraints',
    description: 'Checks if output format/length is specified',
    check: (prompt: string) => {
      const hasConstraints = /format|word count|length|limit|bullet points|paragraphs|steps|examples/i.test(prompt)
      if (!hasConstraints) {
        return {
          passed: false,
          message: 'No output constraints specified. Consider adding format requirements or length guidelines.',
          severity: 'warning'
        }
      }
      return { passed: true, message: 'Prompt includes output constraints.', severity: 'success' }
    }
  },
  {
    id: 'jargon',
    name: 'Technical Jargon',
    description: 'Checks for excessive technical jargon',
    check: (prompt: string) => {
      // This is a simplified jargon check - in a real implementation, 
      // we'd have a more comprehensive list or use an NLP model
      const jargonCount = (prompt.match(/technical|algorithm|framework|implementation|methodology|paradigm|infrastructure|architecture|interface|integration/g) || []).length
      const wordCount = prompt.split(/\s+/).length
      
      if (jargonCount > 5 && jargonCount / wordCount > 0.1) {
        return {
          passed: false,
          message: 'Prompt contains a high amount of technical jargon. Consider simplifying the language for better results.',
          severity: 'warning'
        }
      }
      return { passed: true, message: 'Prompt uses an appropriate level of technical language.', severity: 'success' }
    }
  },
  {
    id: 'ambiguity',
    name: 'Ambiguity Check',
    description: 'Checks for potentially ambiguous language',
    check: (prompt: string) => {
      const ambiguousTerms = /it|they|this|that|those|these|there|their|some|many|few|several|various|different|like|etc\./gi
      const matches = prompt.match(ambiguousTerms) || []
      
      if (matches.length > 3) {
        return {
          passed: false,
          message: 'Prompt contains potentially ambiguous terms. Be more specific about what you\'re referring to.',
          severity: 'warning'
        }
      }
      return { passed: true, message: 'Prompt uses clear, specific language.', severity: 'success' }
    }
  }
]

// Generate improvement suggestions based on failed rules
function generateImprovedPrompt(prompt: string, results: { rule: PromptRule, result: RuleResult }[]): string {
  const failedResults = results.filter(r => !r.result.passed)
  
  if (failedResults.length === 0) {
    return prompt
  }
  
  const improvements = failedResults
    .map(r => r.result.message)
    .join(' ')
  
  return `${prompt}\n\n# Additional guidelines to consider:\n- ${improvements.split('. ').join('\n- ')}`
}

export async function POST(request: Request) {
  try {
    const { prompt } = await request.json()
    
    if (!prompt || typeof prompt !== 'string') {
      return NextResponse.json(
        { error: 'Invalid request. Please provide a prompt string.' },
        { status: 400 }
      )
    }
    
    // Analyze the prompt with each rule
    const results = promptRules.map(rule => ({
      rule,
      result: rule.check(prompt)
    }))
    
    // Calculate overall score
    const passedRules = results.filter(r => r.result.passed).length
    const score = Math.round((passedRules / promptRules.length) * 100)
    
    // Generate improved prompt
    const improvedPrompt = generateImprovedPrompt(prompt, results)
    
    return NextResponse.json({
      score,
      results: results.map(r => ({
        id: r.rule.id,
        name: r.rule.name,
        description: r.rule.description,
        result: r.result
      })),
      improvedPrompt: improvedPrompt
    })
  } catch (error) {
    console.error('Error analyzing prompt:', error)
    return NextResponse.json(
      { error: 'Error analyzing prompt' },
      { status: 500 }
    )
  }
} 