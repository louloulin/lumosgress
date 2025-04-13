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
import { AlertCircle, CheckCircle, Info, Zap, DollarSign, Clock, Gauge } from 'lucide-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'

// Sample API request templates
const requestTemplates = [
  { 
    name: 'OpenAI Chat Completion',
    model: 'gpt-4',
    request: JSON.stringify({
      model: "gpt-4",
      messages: [
        { role: "system", content: "You are a helpful assistant." },
        { role: "user", content: "Explain quantum computing in simple terms" }
      ],
      temperature: 0.7,
      max_tokens: 150
    }, null, 2)
  },
  { 
    name: 'OpenAI Function Call',
    model: 'gpt-4',
    request: JSON.stringify({
      model: "gpt-4",
      messages: [
        { role: "system", content: "You are a helpful assistant." },
        { role: "user", content: "What's the weather like in Boston?" }
      ],
      functions: [
        {
          name: "get_weather",
          description: "Get the current weather in a given location",
          parameters: {
            type: "object",
            properties: {
              location: {
                type: "string",
                description: "The city and state, e.g. San Francisco, CA"
              },
              unit: {
                type: "string",
                enum: ["celsius", "fahrenheit"],
                description: "The temperature unit to use"
              }
            },
            required: ["location"]
          }
        }
      ],
      function_call: "auto"
    }, null, 2)
  },
  { 
    name: 'Anthropic Completion',
    model: 'claude-3-opus',
    request: JSON.stringify({
      model: "claude-3-opus",
      messages: [
        { role: "user", content: "Write a summary of the main quantum computing concepts" }
      ],
      temperature: 0.5,
      max_tokens: 200
    }, null, 2)
  }
]

// Models available for analysis
const modelOptions = [
  { value: 'gpt-4', label: 'GPT-4', provider: 'OpenAI' },
  { value: 'gpt-3.5-turbo', label: 'GPT-3.5 Turbo', provider: 'OpenAI' },
  { value: 'claude-3-opus', label: 'Claude 3 Opus', provider: 'Anthropic' },
  { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet', provider: 'Anthropic' },
  { value: 'gemini-pro', label: 'Gemini Pro', provider: 'Google' },
  { value: 'llama-3-70b', label: 'Llama 3 70B', provider: 'Meta' }
]

// Types for analysis results
type AnalysisSeverity = 'error' | 'warning' | 'success' | 'info'

interface AnalysisResult {
  passed: boolean
  message: string
  severity: AnalysisSeverity
  details?: string
}

interface AnalysisItem {
  id: string
  category: string
  name: string
  description: string
  result: AnalysisResult
}

interface RequestAnalysisResponse {
  score: number
  results: AnalysisItem[]
  optimizedRequest: string
  estimatedCost: number
  estimatedResponseTime: number
  tokenUsage: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  }
}

export default function AIRequestAnalyzerPage() {
  const [selectedModel, setSelectedModel] = useState('gpt-4')
  const [requestBody, setRequestBody] = useState('')
  const [analysisResults, setAnalysisResults] = useState<AnalysisItem[]>([])
  const [overallScore, setOverallScore] = useState<number | null>(null)
  const [isAnalyzing, setIsAnalyzing] = useState(false)
  const [optimizedRequest, setOptimizedRequest] = useState('')
  const [activeTab, setActiveTab] = useState('original')
  const [error, setError] = useState<string | null>(null)
  const [estimatedCost, setEstimatedCost] = useState<number | null>(null)
  const [estimatedResponseTime, setEstimatedResponseTime] = useState<number | null>(null)
  const [tokenUsage, setTokenUsage] = useState<{promptTokens: number, completionTokens: number, totalTokens: number} | null>(null)
  const [categoryFilters, setCategoryFilters] = useState<string[]>([])

  const analyzeRequest = async () => {
    if (!requestBody.trim()) return
    
    setIsAnalyzing(true)
    setOptimizedRequest('')
    setError(null)
    
    try {
      // Mock analysis for demonstration purposes
      // In a real implementation, this would call an actual API endpoint
      await new Promise(resolve => setTimeout(resolve, 1500))
      
      const mockResponse: RequestAnalysisResponse = {
        score: 78,
        estimatedCost: 0.0125,
        estimatedResponseTime: 2.3,
        tokenUsage: {
          promptTokens: 320,
          completionTokens: 150,
          totalTokens: 470
        },
        results: [
          {
            id: "1",
            category: "Prompt Quality",
            name: "System Message",
            description: "Checks if the system message is clear and specific",
            result: {
              passed: true,
              message: "System message is clear and provides good context",
              severity: "success"
            }
          },
          {
            id: "2",
            category: "Parameters",
            name: "Temperature Setting",
            description: "Analyzes temperature parameter for appropriateness",
            result: {
              passed: false,
              message: "Temperature of 0.7 may be too high for factual responses",
              severity: "warning",
              details: "For fact-based tasks like explanations, a lower temperature (0.1-0.3) produces more consistent results."
            }
          },
          {
            id: "3",
            category: "Parameters",
            name: "Max Tokens",
            description: "Checks if max tokens is appropriate for the task",
            result: {
              passed: false,
              message: "Max tokens (150) is likely too low for a comprehensive explanation",
              severity: "error",
              details: "For explanations of complex topics like quantum computing, consider at least 500-1000 tokens."
            }
          },
          {
            id: "4",
            category: "Model Selection",
            name: "Model Appropriateness",
            description: "Evaluates if the selected model is suitable for this task",
            result: {
              passed: true,
              message: "GPT-4 is well-suited for explanations of complex topics",
              severity: "success"
            }
          },
          {
            id: "5",
            category: "Cost Efficiency",
            name: "Cost vs Capability",
            description: "Analyzes if a more cost-effective model could achieve similar results",
            result: {
              passed: false,
              message: "Consider using GPT-3.5 Turbo for simple explanations to reduce costs",
              severity: "info",
              details: "For basic explanations, GPT-3.5 Turbo could provide satisfactory results at roughly 1/10th the cost."
            }
          },
          {
            id: "6",
            category: "Request Structure",
            name: "JSON Format",
            description: "Validates the JSON structure of the request",
            result: {
              passed: true,
              message: "Request structure is valid and well-formatted",
              severity: "success"
            }
          },
          {
            id: "7",
            category: "Prompt Quality",
            name: "Prompt Clarity",
            description: "Evaluates if the prompt is clear and specific enough",
            result: {
              passed: true,
              message: "Prompt is clear and specific about what is needed",
              severity: "success"
            }
          }
        ],
        optimizedRequest: JSON.stringify({
          model: "gpt-4",
          messages: [
            { 
              role: "system", 
              content: "You are a helpful assistant that explains complex topics clearly and accurately using simple language and analogies when appropriate." 
            },
            { 
              role: "user", 
              content: "Explain quantum computing in simple terms. Include the basic principles, how it differs from classical computing, and 1-2 potential applications."
            }
          ],
          temperature: 0.3,
          max_tokens: 500
        }, null, 2)
      };
      
      setAnalysisResults(mockResponse.results)
      setOverallScore(mockResponse.score)
      setOptimizedRequest(mockResponse.optimizedRequest)
      setEstimatedCost(mockResponse.estimatedCost)
      setEstimatedResponseTime(mockResponse.estimatedResponseTime)
      setTokenUsage(mockResponse.tokenUsage)
      setActiveTab('analysis')
      
      // Extract unique categories for filtering
      const categories = Array.from(new Set(mockResponse.results.map(item => item.category)))
      setCategoryFilters(categories)
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unknown error occurred')
      console.error('Error analyzing request:', err)
    } finally {
      setIsAnalyzing(false)
    }
  }

  const applyTemplate = (template: typeof requestTemplates[0]) => {
    setSelectedModel(template.model)
    setRequestBody(template.request)
  }

  const filteredResults = categoryFilters.length > 0 
    ? analysisResults.filter(item => categoryFilters.includes(item.category))
    : analysisResults

  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div>
          <h1 className="text-3xl font-bold">AI Request Analyzer</h1>
          <p className="text-muted-foreground">
            Analyze and optimize your AI API requests for better performance and cost efficiency
          </p>
        </div>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Request Configuration</CardTitle>
              <CardDescription>
                Enter your API request JSON to analyze for quality, efficiency and cost
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div>
                  <label className="text-sm font-medium">Templates</label>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {requestTemplates.map((template, index) => (
                      <Button 
                        key={index} 
                        variant="outline" 
                        size="sm"
                        onClick={() => applyTemplate(template)}
                      >
                        {template.name}
                      </Button>
                    ))}
                  </div>
                </div>
                
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label htmlFor="model">Model</Label>
                      <Select 
                        value={selectedModel} 
                        onValueChange={setSelectedModel}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select a model" />
                        </SelectTrigger>
                        <SelectContent>
                          {modelOptions.map(model => (
                            <SelectItem key={model.value} value={model.value}>
                              {model.label} ({model.provider})
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="request">Request JSON</Label>
                  <Textarea
                    id="request"
                    value={requestBody}
                    onChange={(e) => setRequestBody(e.target.value)}
                    placeholder='{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello world"}]}'
                    className="min-h-[300px] font-mono text-sm"
                  />
                </div>
              </div>
            </CardContent>
            <CardFooter>
              <Button onClick={analyzeRequest} disabled={!requestBody.trim() || isAnalyzing}>
                {isAnalyzing ? 'Analyzing...' : 'Analyze Request'}
              </Button>
            </CardFooter>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Analysis Results</CardTitle>
              <CardDescription>
                Detailed feedback on your request quality, efficiency and cost
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
                  <TabsList className="grid w-full grid-cols-3">
                    <TabsTrigger value="analysis">Analysis</TabsTrigger>
                    <TabsTrigger value="metrics">Metrics</TabsTrigger>
                    <TabsTrigger value="optimized">Optimized Request</TabsTrigger>
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
                    
                    <div className="flex flex-wrap gap-2 mb-4">
                      {categoryFilters.map(category => (
                        <Button 
                          key={category}
                          variant="outline" 
                          size="sm"
                          onClick={() => {
                            setCategoryFilters(currentFilters => 
                              currentFilters.includes(category) 
                                ? currentFilters.filter(c => c !== category)
                                : [...currentFilters, category]
                            )
                          }}
                          className={categoryFilters.includes(category) ? "bg-primary text-primary-foreground" : ""}
                        >
                          {category}
                        </Button>
                      ))}
                    </div>
                    
                    <ScrollArea className="h-[400px] pr-4">
                      <Accordion type="multiple" className="w-full">
                        {filteredResults.map((item) => (
                          <AccordionItem key={item.id} value={item.id}>
                            <AccordionTrigger className="hover:no-underline">
                              <div className="flex items-center gap-2">
                                {item.result.severity === 'error' && <AlertCircle className="h-4 w-4 text-red-500" />}
                                {item.result.severity === 'warning' && <Info className="h-4 w-4 text-amber-500" />}
                                {item.result.severity === 'success' && <CheckCircle className="h-4 w-4 text-green-500" />}
                                {item.result.severity === 'info' && <Info className="h-4 w-4 text-blue-500" />}
                                <span>{item.name}</span>
                                <span className="text-xs text-muted-foreground ml-2">{item.category}</span>
                              </div>
                            </AccordionTrigger>
                            <AccordionContent>
                              <div className="pl-6">
                                <p className="text-sm text-muted-foreground mb-2">{item.description}</p>
                                <Alert variant={item.result.passed ? "default" : "destructive"}>
                                  <AlertTitle>{item.result.passed ? "Passed" : "Improvement Needed"}</AlertTitle>
                                  <AlertDescription>{item.result.message}</AlertDescription>
                                </Alert>
                                {item.result.details && (
                                  <p className="text-sm mt-2 px-4 py-2 bg-muted rounded-md">{item.result.details}</p>
                                )}
                              </div>
                            </AccordionContent>
                          </AccordionItem>
                        ))}
                      </Accordion>
                    </ScrollArea>
                  </TabsContent>
                  
                  <TabsContent value="metrics" className="space-y-4 mt-4">
                    <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                      <Card>
                        <CardHeader className="pb-2">
                          <div className="flex items-center justify-between">
                            <CardTitle className="text-sm font-medium">Estimated Cost</CardTitle>
                            <DollarSign className="h-4 w-4 text-muted-foreground" />
                          </div>
                        </CardHeader>
                        <CardContent>
                          <div className="text-2xl font-bold">${estimatedCost?.toFixed(4)}</div>
                          <p className="text-xs text-muted-foreground">Per request</p>
                        </CardContent>
                      </Card>
                      
                      <Card>
                        <CardHeader className="pb-2">
                          <div className="flex items-center justify-between">
                            <CardTitle className="text-sm font-medium">Response Time</CardTitle>
                            <Clock className="h-4 w-4 text-muted-foreground" />
                          </div>
                        </CardHeader>
                        <CardContent>
                          <div className="text-2xl font-bold">{estimatedResponseTime?.toFixed(1)}s</div>
                          <p className="text-xs text-muted-foreground">Estimated average</p>
                        </CardContent>
                      </Card>
                      
                      <Card>
                        <CardHeader className="pb-2">
                          <div className="flex items-center justify-between">
                            <CardTitle className="text-sm font-medium">Token Usage</CardTitle>
                            <Gauge className="h-4 w-4 text-muted-foreground" />
                          </div>
                        </CardHeader>
                        <CardContent>
                          <div className="text-2xl font-bold">{tokenUsage?.totalTokens}</div>
                          <p className="text-xs text-muted-foreground">
                            {tokenUsage?.promptTokens} prompt + {tokenUsage?.completionTokens} completion
                          </p>
                        </CardContent>
                      </Card>
                    </div>
                    
                    <Card>
                      <CardHeader>
                        <CardTitle className="text-sm font-medium">Cost Optimization</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-4">
                          <div>
                            <div className="flex items-center justify-between mb-2">
                              <Label>Alternative Models</Label>
                              <span className="text-xs text-muted-foreground">Cost vs Performance</span>
                            </div>
                            <div className="space-y-2">
                              <div className="flex items-center justify-between bg-secondary/30 p-3 rounded-md">
                                <div>
                                  <p className="font-medium">GPT-4</p>
                                  <p className="text-xs text-muted-foreground">Higher quality, more expensive</p>
                                </div>
                                <div>
                                  <p className="font-medium">$0.0125</p>
                                  <p className="text-xs text-muted-foreground">Current selection</p>
                                </div>
                              </div>
                              
                              <div className="flex items-center justify-between bg-muted p-3 rounded-md">
                                <div>
                                  <p className="font-medium">GPT-3.5 Turbo</p>
                                  <p className="text-xs text-muted-foreground">Good quality, more affordable</p>
                                </div>
                                <div>
                                  <p className="font-medium">$0.0015</p>
                                  <p className="text-xs text-green-500">88% savings</p>
                                </div>
                              </div>
                              
                              <div className="flex items-center justify-between bg-muted p-3 rounded-md">
                                <div>
                                  <p className="font-medium">Claude 3 Sonnet</p>
                                  <p className="text-xs text-muted-foreground">Comparable quality, different provider</p>
                                </div>
                                <div>
                                  <p className="font-medium">$0.0065</p>
                                  <p className="text-xs text-green-500">48% savings</p>
                                </div>
                              </div>
                            </div>
                          </div>
                          
                          <div>
                            <div className="flex items-center justify-between mb-2">
                              <Label>Parameter Optimizations</Label>
                              <span className="text-xs text-muted-foreground">Potential Impact</span>
                            </div>
                            <div className="space-y-1">
                              <div className="flex items-center justify-between py-2">
                                <p className="text-sm">Reduce temperature to 0.3</p>
                                <Badge className="bg-green-100 text-green-800">High Impact</Badge>
                              </div>
                              <div className="flex items-center justify-between py-2">
                                <p className="text-sm">Set max_tokens to 500 (more appropriate for task)</p>
                                <Badge className="bg-amber-100 text-amber-800">Medium Impact</Badge>
                              </div>
                              <div className="flex items-center justify-between py-2">
                                <p className="text-sm">Add a more specific system message</p>
                                <Badge className="bg-blue-100 text-blue-800">Quality Impact</Badge>
                              </div>
                            </div>
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>
                  
                  <TabsContent value="optimized" className="space-y-4 mt-4">
                    <Alert>
                      <Zap className="h-4 w-4" />
                      <AlertTitle>Optimized Request</AlertTitle>
                      <AlertDescription>
                        This optimized version improves quality, efficiency, and may reduce costs.
                      </AlertDescription>
                    </Alert>
                    
                    <Textarea
                      value={optimizedRequest}
                      readOnly
                      className="min-h-[400px] font-mono text-sm"
                    />
                    
                    <div className="flex justify-end">
                      <Button 
                        variant="outline"
                        onClick={() => {
                          navigator.clipboard.writeText(optimizedRequest);
                        }}
                      >
                        Copy to Clipboard
                      </Button>
                    </div>
                  </TabsContent>
                </Tabs>
              ) : (
                <div className="flex flex-col items-center justify-center py-10 text-center">
                  <div className="rounded-full bg-muted p-3 mb-4">
                    <Zap className="h-6 w-6 text-muted-foreground" />
                  </div>
                  <h3 className="font-semibold mb-2">No Analysis Yet</h3>
                  <p className="text-sm text-muted-foreground max-w-md">
                    Enter your AI API request and click "Analyze Request" to get detailed feedback on quality, efficiency, and cost.
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