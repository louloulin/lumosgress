"use client"

import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"

export default function DeveloperPortalPage() {
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <h1 className="text-3xl font-bold">Developer Portal</h1>
        <p className="text-muted-foreground">
          Resources and documentation for integrating with the Proksi AI Gateway.
        </p>
        
        <Tabs defaultValue="getting-started" className="mt-6">
          <TabsList>
            <TabsTrigger value="getting-started">Getting Started</TabsTrigger>
            <TabsTrigger value="documentation">Documentation</TabsTrigger>
            <TabsTrigger value="sdk">SDK & Libraries</TabsTrigger>
            <TabsTrigger value="api-reference">API Reference</TabsTrigger>
            <TabsTrigger value="examples">Examples</TabsTrigger>
          </TabsList>
          
          <TabsContent value="getting-started" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Welcome to Proksi AI Gateway</CardTitle>
                <CardDescription>
                  Everything you need to get started with integrating our AI Gateway
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  <Card>
                    <CardHeader className="pb-2">
                      <CardTitle className="text-lg">Quick Start</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <p className="text-sm">Get up and running with Proksi in minutes</p>
                    </CardContent>
                    <CardFooter>
                      <Button variant="outline" size="sm">View Guide</Button>
                    </CardFooter>
                  </Card>
                  
                  <Card>
                    <CardHeader className="pb-2">
                      <CardTitle className="text-lg">SDK Installation</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <p className="text-sm">Install and set up the official Proksi SDK</p>
                    </CardContent>
                    <CardFooter>
                      <Button variant="outline" size="sm">Installation Guide</Button>
                    </CardFooter>
                  </Card>
                  
                  <Card>
                    <CardHeader className="pb-2">
                      <CardTitle className="text-lg">Authentication</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <p className="text-sm">Learn how to authenticate with the API</p>
                    </CardContent>
                    <CardFooter>
                      <Button variant="outline" size="sm">Auth Guide</Button>
                    </CardFooter>
                  </Card>
                </div>
                
                <div className="pt-4">
                  <h3 className="text-xl font-semibold mb-2">Core Concepts</h3>
                  <p className="text-sm mb-4">
                    Understand the key concepts and architecture of the Proksi AI Gateway
                  </p>
                  
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="flex flex-col space-y-2 rounded-lg border p-4">
                      <h4 className="font-medium">LLM Routing</h4>
                      <p className="text-sm text-muted-foreground">Route requests to different LLM providers based on content and requirements</p>
                      <Button variant="link" size="sm" className="self-start px-0">Learn more</Button>
                    </div>
                    
                    <div className="flex flex-col space-y-2 rounded-lg border p-4">
                      <h4 className="font-medium">Prompt Transformation</h4>
                      <p className="text-sm text-muted-foreground">Enhance and standardize prompts before they reach LLM providers</p>
                      <Button variant="link" size="sm" className="self-start px-0">Learn more</Button>
                    </div>
                    
                    <div className="flex flex-col space-y-2 rounded-lg border p-4">
                      <h4 className="font-medium">Vector Operations</h4>
                      <p className="text-sm text-muted-foreground">Store, retrieve, and search vector embeddings with multiple database options</p>
                      <Button variant="link" size="sm" className="self-start px-0">Learn more</Button>
                    </div>
                    
                    <div className="flex flex-col space-y-2 rounded-lg border p-4">
                      <h4 className="font-medium">AI Security</h4>
                      <p className="text-sm text-muted-foreground">Protect your AI applications with content filtering and rate limiting</p>
                      <Button variant="link" size="sm" className="self-start px-0">Learn more</Button>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="documentation" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Documentation</CardTitle>
                <CardDescription>
                  Comprehensive guides and tutorials for using Proksi AI Gateway
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-6">
                  <div>
                    <h3 className="text-lg font-semibold mb-2">Guides</h3>
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                      <Card>
                        <CardHeader className="pb-2">
                          <CardTitle className="text-base">Installation & Setup</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                          <ul className="text-sm space-y-1 list-disc list-inside text-muted-foreground">
                            <li>Docker Installation</li>
                            <li>Kubernetes Deployment</li>
                            <li>Configuration Options</li>
                          </ul>
                        </CardContent>
                        <CardFooter>
                          <Button variant="ghost" size="sm">View Guides</Button>
                        </CardFooter>
                      </Card>
                      
                      <Card>
                        <CardHeader className="pb-2">
                          <CardTitle className="text-base">Plugin Development</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                          <ul className="text-sm space-y-1 list-disc list-inside text-muted-foreground">
                            <li>Plugin Architecture</li>
                            <li>Creating Custom Plugins</li>
                            <li>WASM Integration</li>
                          </ul>
                        </CardContent>
                        <CardFooter>
                          <Button variant="ghost" size="sm">View Guides</Button>
                        </CardFooter>
                      </Card>
                      
                      <Card>
                        <CardHeader className="pb-2">
                          <CardTitle className="text-base">Advanced Features</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                          <ul className="text-sm space-y-1 list-disc list-inside text-muted-foreground">
                            <li>Multi-Model Aggregation</li>
                            <li>Semantic Routing</li>
                            <li>Advanced Caching</li>
                          </ul>
                        </CardContent>
                        <CardFooter>
                          <Button variant="ghost" size="sm">View Guides</Button>
                        </CardFooter>
                      </Card>
                    </div>
                  </div>
                  
                  <div>
                    <h3 className="text-lg font-semibold mb-2">Tutorials</h3>
                    <div className="grid gap-4 md:grid-cols-2">
                      <div className="flex items-start space-x-4 rounded-lg border p-4">
                        <div className="bg-primary text-primary-foreground rounded-md p-2">
                          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="w-5 h-5">
                            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                            <polyline points="14 2 14 8 20 8"/>
                          </svg>
                        </div>
                        <div className="flex-1">
                          <h4 className="font-medium">Building a RAG Application</h4>
                          <p className="text-sm text-muted-foreground mt-1">Learn how to build a complete Retrieval Augmented Generation app using Proksi</p>
                          <Badge variant="outline" className="mt-2">Beginner</Badge>
                        </div>
                      </div>
                      
                      <div className="flex items-start space-x-4 rounded-lg border p-4">
                        <div className="bg-primary text-primary-foreground rounded-md p-2">
                          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="w-5 h-5">
                            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                            <polyline points="14 2 14 8 20 8"/>
                          </svg>
                        </div>
                        <div className="flex-1">
                          <h4 className="font-medium">Cost Optimization Strategies</h4>
                          <p className="text-sm text-muted-foreground mt-1">Optimize your AI costs with smart routing and caching techniques</p>
                          <Badge variant="outline" className="mt-2">Advanced</Badge>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="sdk" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>SDK and Client Libraries</CardTitle>
                <CardDescription>
                  Official client libraries for different programming languages
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-8">
                  <div>
                    <div className="flex items-center justify-between mb-4">
                      <h3 className="text-lg font-semibold">JavaScript/TypeScript SDK</h3>
                      <Badge variant="outline" className="text-green-600 bg-green-50">Latest: v0.1.0</Badge>
                    </div>
                    
                    <div className="mb-4 rounded-md bg-muted p-4">
                      <pre className="text-sm overflow-x-auto"><code>npm install proksi-sdk</code></pre>
                    </div>
                    
                    <div className="mb-4 rounded-md border p-4">
                      <h4 className="font-medium mb-2">Quick Example</h4>
                      <pre className="text-sm overflow-x-auto"><code>{`import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://your-proksi-instance.com',
  apiKey: 'your-api-key'
});

// Send a completion request
const response = await client.completion({
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Tell me about AI gateways.' }
  ]
});

console.log(response.choices[0].message.content);`}</code></pre>
                    </div>
                    
                    <div className="flex items-center justify-between">
                      <Button variant="outline">Documentation</Button>
                      <Button variant="outline">API Reference</Button>
                      <Button variant="outline">Examples</Button>
                    </div>
                  </div>
                  
                  <div className="grid gap-4 md:grid-cols-2">
                    <Card>
                      <CardHeader>
                        <div className="flex items-center justify-between">
                          <CardTitle>Python SDK</CardTitle>
                          <Badge variant="secondary">Coming Soon</Badge>
                        </div>
                      </CardHeader>
                      <CardContent>
                        <p className="text-sm text-muted-foreground">
                          Python client library for the Proksi AI Gateway, optimized for data science and ML workflows.
                        </p>
                      </CardContent>
                      <CardFooter>
                        <Button variant="outline" disabled>Join Waitlist</Button>
                      </CardFooter>
                    </Card>
                    
                    <Card>
                      <CardHeader>
                        <div className="flex items-center justify-between">
                          <CardTitle>Go Client</CardTitle>
                          <Badge variant="secondary">Coming Soon</Badge>
                        </div>
                      </CardHeader>
                      <CardContent>
                        <p className="text-sm text-muted-foreground">
                          Go client library for the Proksi AI Gateway, ideal for backend and microservice integrations.
                        </p>
                      </CardContent>
                      <CardFooter>
                        <Button variant="outline" disabled>Join Waitlist</Button>
                      </CardFooter>
                    </Card>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="api-reference" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>API Reference</CardTitle>
                <CardDescription>
                  Detailed API reference documentation
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-6">
                  <div className="mb-4">
                    <h3 className="text-lg font-semibold mb-2">Base URL</h3>
                    <div className="rounded-md bg-muted p-4">
                      <pre className="text-sm"><code>https://your-proksi-instance.com</code></pre>
                    </div>
                  </div>
                  
                  <div className="mb-4">
                    <h3 className="text-lg font-semibold mb-2">Authentication</h3>
                    <p className="text-sm text-muted-foreground mb-4">
                      All API requests require authentication using an API key provided in the request header.
                    </p>
                    <div className="rounded-md bg-muted p-4">
                      <pre className="text-sm"><code>Authorization: Bearer YOUR_API_KEY</code></pre>
                    </div>
                  </div>
                  
                  <div>
                    <h3 className="text-lg font-semibold mb-4">Endpoints</h3>
                    
                    <div className="space-y-4">
                      <div className="rounded-lg border p-4">
                        <div className="flex items-center gap-2 mb-2">
                          <Badge className="bg-blue-500">POST</Badge>
                          <h4 className="font-medium">/v1/chat/completions</h4>
                        </div>
                        <p className="text-sm text-muted-foreground mb-2">
                          Generate a completion from a chat conversation
                        </p>
                        <Button variant="outline" size="sm">View Documentation</Button>
                      </div>
                      
                      <div className="rounded-lg border p-4">
                        <div className="flex items-center gap-2 mb-2">
                          <Badge className="bg-blue-500">POST</Badge>
                          <h4 className="font-medium">/v1/completions</h4>
                        </div>
                        <p className="text-sm text-muted-foreground mb-2">
                          Generate a completion from a prompt
                        </p>
                        <Button variant="outline" size="sm">View Documentation</Button>
                      </div>
                      
                      <div className="rounded-lg border p-4">
                        <div className="flex items-center gap-2 mb-2">
                          <Badge className="bg-blue-500">POST</Badge>
                          <h4 className="font-medium">/vectors/upsert</h4>
                        </div>
                        <p className="text-sm text-muted-foreground mb-2">
                          Insert or update vectors in the vector database
                        </p>
                        <Button variant="outline" size="sm">View Documentation</Button>
                      </div>
                      
                      <div className="rounded-lg border p-4">
                        <div className="flex items-center gap-2 mb-2">
                          <Badge className="bg-blue-500">POST</Badge>
                          <h4 className="font-medium">/vectors/search</h4>
                        </div>
                        <p className="text-sm text-muted-foreground mb-2">
                          Search for similar vectors in the vector database
                        </p>
                        <Button variant="outline" size="sm">View Documentation</Button>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <Button>View Full API Reference</Button>
              </CardFooter>
            </Card>
          </TabsContent>
          
          <TabsContent value="examples" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Example Projects</CardTitle>
                <CardDescription>
                  Ready-to-use example projects and code snippets
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-6">
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    <Card>
                      <CardHeader className="pb-2">
                        <div className="flex justify-between items-start">
                          <CardTitle className="text-base">Chat Application</CardTitle>
                          <Badge>TypeScript</Badge>
                        </div>
                      </CardHeader>
                      <CardContent className="pt-0">
                        <p className="text-sm text-muted-foreground mb-2">
                          A simple chat application using the Proksi SDK with Next.js
                        </p>
                        <div className="flex gap-2 mt-4">
                          <Button variant="outline" size="sm">View Demo</Button>
                          <Button variant="outline" size="sm">Source Code</Button>
                        </div>
                      </CardContent>
                    </Card>
                    
                    <Card>
                      <CardHeader className="pb-2">
                        <div className="flex justify-between items-start">
                          <CardTitle className="text-base">Vector Search Demo</CardTitle>
                          <Badge>JavaScript</Badge>
                        </div>
                      </CardHeader>
                      <CardContent className="pt-0">
                        <p className="text-sm text-muted-foreground mb-2">
                          Demonstrates semantic search capabilities using vector embeddings
                        </p>
                        <div className="flex gap-2 mt-4">
                          <Button variant="outline" size="sm">View Demo</Button>
                          <Button variant="outline" size="sm">Source Code</Button>
                        </div>
                      </CardContent>
                    </Card>
                    
                    <Card>
                      <CardHeader className="pb-2">
                        <div className="flex justify-between items-start">
                          <CardTitle className="text-base">Model Comparison</CardTitle>
                          <Badge>React</Badge>
                        </div>
                      </CardHeader>
                      <CardContent className="pt-0">
                        <p className="text-sm text-muted-foreground mb-2">
                          Compare responses from different LLM providers side by side
                        </p>
                        <div className="flex gap-2 mt-4">
                          <Button variant="outline" size="sm">View Demo</Button>
                          <Button variant="outline" size="sm">Source Code</Button>
                        </div>
                      </CardContent>
                    </Card>
                  </div>
                  
                  <div>
                    <h3 className="text-lg font-semibold mb-2">Code Snippets</h3>
                    <div className="space-y-4">
                      <div className="rounded-lg border p-4">
                        <h4 className="font-medium mb-2">Streaming Completions</h4>
                        <pre className="text-sm overflow-x-auto bg-muted rounded-md p-3"><code>{`import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://your-proksi-instance.com',
  apiKey: 'your-api-key'
});

// Stream completions
const stream = await client.streamCompletion({
  messages: [
    { role: 'user', content: 'Write a short story about AI' }
  ]
});

for await (const chunk of stream) {
  process.stdout.write(chunk.choices[0]?.delta?.content || '');
}`}</code></pre>
                      </div>
                      
                      <div className="rounded-lg border p-4">
                        <h4 className="font-medium mb-2">Vector Database Operations</h4>
                        <pre className="text-sm overflow-x-auto bg-muted rounded-md p-3"><code>{`import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://your-proksi-instance.com',
  apiKey: 'your-api-key'
});

// Upsert vectors
await client.upsertVectors({
  vectors: [
    { 
      id: 'doc1', 
      values: [0.1, 0.2, 0.3, ...], 
      metadata: { source: 'article', title: 'AI Basics' } 
    }
  ]
});

// Search vectors
const results = await client.searchVectors({
  queryVector: [0.2, 0.3, 0.4, ...],
  topK: 5
});`}</code></pre>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
} 