'use client'

import { useState } from 'react'
import { DashboardLayout } from '@/components/layout/dashboard-layout'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { AlertCircle, CheckCircle, Info } from 'lucide-react'

// Default prompt template examples
const promptTemplates = [
  { name: 'General Instruction', text: 'You are an AI assistant. Be concise, accurate, and helpful.' },
  { name: 'Creative Writing', text: 'Write a creative story about [topic]. Include vivid descriptions and compelling characters.' },
  { name: 'Technical Explanation', text: 'Explain [technical concept] in simple terms that a beginner can understand.' },
  { name: 'Data Analysis', text: 'Analyze the following data: [data]. Identify trends, patterns, and provide insights.' }
]

// Types for rule results
type RuleSeverity = 'error' | 'warning' | 'success'

interface RuleResult {
  passed: boolean
  message: string
  severity: RuleSeverity
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

export default function PromptDebuggerPage() {
  const [prompt, setPrompt] = useState('')
  const [analysisResults, setAnalysisResults] = useState<AnalysisRule[]>([])
  const [overallScore, setOverallScore] = useState<number | null>(null)
  const [isAnalyzing, setIsAnalyzing] = useState(false)
  const [improvedPrompt, setImprovedPrompt] = useState('')
  const [activeTab, setActiveTab] = useState('original')
  const [error, setError] = useState<string | null>(null)

  const analyzePrompt = async () => {
    if (!prompt.trim()) return
    
    setIsAnalyzing(true)
    setImprovedPrompt('')
    setError(null)
    
    try {
      const response = await fetch('/api/prompt-debugger', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ prompt }),
      })
      
      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Failed to analyze prompt')
      }
      
      const data: AnalysisResponse = await response.json()
      setAnalysisResults(data.results)
      setOverallScore(data.score)
      setImprovedPrompt(data.improvedPrompt)
      setActiveTab('analysis')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unknown error occurred')
      console.error('Error analyzing prompt:', err)
    } finally {
      setIsAnalyzing(false)
    }
  }

  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div>
          <h1 className="text-3xl font-bold">Prompt Debugger</h1>
          <p className="text-muted-foreground">
            Analyze and optimize your prompts for better LLM responses
          </p>
        </div>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Input Prompt</CardTitle>
              <CardDescription>
                Enter your prompt to analyze for quality and effectiveness
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div>
                  <label className="text-sm font-medium">Templates</label>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {promptTemplates.map((template, index) => (
                      <Button 
                        key={index} 
                        variant="outline" 
                        size="sm"
                        onClick={() => setPrompt(template.text)}
                      >
                        {template.name}
                      </Button>
                    ))}
                  </div>
                </div>
                <Textarea
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  placeholder="Enter your prompt here..."
                  className="min-h-[300px] font-mono text-sm"
                />
              </div>
            </CardContent>
            <CardFooter>
              <Button onClick={analyzePrompt} disabled={!prompt.trim() || isAnalyzing}>
                {isAnalyzing ? 'Analyzing...' : 'Analyze Prompt'}
              </Button>
            </CardFooter>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Analysis Results</CardTitle>
              <CardDescription>
                Detailed feedback on your prompt quality
              </CardDescription>
            </CardHeader>
            <CardContent>
              {error && (
                <Alert variant="destructive" className="mb-4">
                  <AlertCircle className="h-4 w-4" />
                  <AlertTitle>Error</AlertTitle>
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}
              
              {analysisResults.length > 0 ? (
                <Tabs value={activeTab} onValueChange={setActiveTab}>
                  <TabsList className="grid w-full grid-cols-2">
                    <TabsTrigger value="analysis">Analysis</TabsTrigger>
                    <TabsTrigger value="improved">Improved Prompt</TabsTrigger>
                  </TabsList>
                  
                  <TabsContent value="analysis" className="space-y-4 mt-4">
                    {overallScore !== null && (
                      <div className="flex items-center justify-between bg-secondary/50 p-4 rounded-lg">
                        <div>
                          <h3 className="font-semibold">Overall Score</h3>
                          <p className="text-sm text-muted-foreground">
                            Based on {analysisResults.length} quality criteria
                          </p>
                        </div>
                        <div className={`text-2xl font-bold ${
                          overallScore >= 80 ? 'text-green-500' : 
                          overallScore >= 50 ? 'text-amber-500' : 'text-red-500'
                        }`}>
                          {overallScore}/100
                        </div>
                      </div>
                    )}
                    
                    <ScrollArea className="h-[400px] pr-4">
                      <Accordion type="multiple" className="w-full">
                        {analysisResults.map((rule) => (
                          <AccordionItem key={rule.id} value={rule.id}>
                            <AccordionTrigger className="hover:no-underline">
                              <div className="flex items-center gap-2">
                                {rule.result.severity === 'error' && <AlertCircle className="h-4 w-4 text-red-500" />}
                                {rule.result.severity === 'warning' && <Info className="h-4 w-4 text-amber-500" />}
                                {rule.result.severity === 'success' && <CheckCircle className="h-4 w-4 text-green-500" />}
                                <span>{rule.name}</span>
                              </div>
                            </AccordionTrigger>
                            <AccordionContent>
                              <div className="pl-6">
                                <p className="text-sm text-muted-foreground mb-2">{rule.description}</p>
                                <Alert variant={rule.result.passed ? "default" : "destructive"}>
                                  <AlertTitle>{rule.result.passed ? "Passed" : "Improvement Needed"}</AlertTitle>
                                  <AlertDescription>{rule.result.message}</AlertDescription>
                                </Alert>
                              </div>
                            </AccordionContent>
                          </AccordionItem>
                        ))}
                      </Accordion>
                    </ScrollArea>
                  </TabsContent>
                  
                  <TabsContent value="improved" className="space-y-4 mt-4">
                    {improvedPrompt ? (
                      <div className="space-y-4">
                        <Alert>
                          <Info className="h-4 w-4" />
                          <AlertTitle>AI-Suggested Improvements</AlertTitle>
                          <AlertDescription>
                            This improved version addresses the issues found in the analysis.
                          </AlertDescription>
                        </Alert>
                        <Textarea
                          value={improvedPrompt}
                          readOnly
                          className="min-h-[400px] font-mono text-sm"
                        />
                        <div className="flex justify-end">
                          <Button 
                            variant="outline" 
                            onClick={() => {
                              setPrompt(improvedPrompt)
                              setActiveTab('original')
                            }}
                          >
                            Use Improved Prompt
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <div className="flex items-center justify-center h-[400px]">
                        <p className="text-muted-foreground">
                          Analyze your prompt to see improvements
                        </p>
                      </div>
                    )}
                  </TabsContent>
                </Tabs>
              ) : (
                <div className="flex items-center justify-center h-[400px]">
                  <p className="text-muted-foreground">
                    Enter a prompt and click "Analyze Prompt" to see results
                  </p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </DashboardLayout>
  )
} 