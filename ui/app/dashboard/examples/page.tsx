'use client'

import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { FolderGit2, Code, GitBranch, Search, Cpu, MessagesSquare, Bot, FileCode } from "lucide-react"

// Example project data
const exampleProjects = [
  {
    id: 'chat-bot',
    title: 'AI Chat Application',
    description: 'A full-featured chat application built with Next.js and Proksi AI Gateway.',
    category: 'application',
    tags: ['nextjs', 'react', 'typescript'],
    level: 'beginner',
    image: '/examples/chat-application.png',
    githubUrl: 'https://github.com/proksi/examples/chat-application',
    demoUrl: 'https://chat-demo.proksi.dev',
    features: [
      'Streaming responses',
      'Chat history',
      'Multiple LLM providers',
      'Prompt templates',
    ]
  },
  {
    id: 'rag-app',
    title: 'RAG Knowledge Base',
    description: 'Retrieval Augmented Generation system with document indexing and semantic search.',
    category: 'application',
    tags: ['vector-db', 'python', 'fastapi'],
    level: 'intermediate',
    image: '/examples/rag-app.png',
    githubUrl: 'https://github.com/proksi/examples/rag-application',
    demoUrl: 'https://rag-demo.proksi.dev',
    features: [
      'Document processing and chunking',
      'Vector embeddings and storage',
      'Semantic search',
      'Context-aware responses',
    ]
  },
  {
    id: 'multi-model',
    title: 'Multi-Model Router',
    description: 'Distribute requests across multiple LLM providers with fallback and load balancing.',
    category: 'integration',
    tags: ['javascript', 'node', 'express'],
    level: 'intermediate',
    image: '/examples/multi-model-router.png',
    githubUrl: 'https://github.com/proksi/examples/multi-model-router',
    demoUrl: 'https://router-demo.proksi.dev',
    features: [
      'Load balancing across providers',
      'Automatic fallback',
      'Response time metrics',
      'Cost optimization',
    ]
  },
  {
    id: 'document-chat',
    title: 'Document Q&A Bot',
    description: 'Chat with any document - PDF, Word, or text - using a vector database and LLMs.',
    category: 'application',
    tags: ['nextjs', 'typescript', 'pinecone'],
    level: 'intermediate',
    image: '/examples/document-chat.png',
    githubUrl: 'https://github.com/proksi/examples/document-chat',
    demoUrl: 'https://docqa-demo.proksi.dev',
    features: [
      'Document upload and processing',
      'Chunk management',
      'Relevance scoring',
      'Citation and sources',
    ]
  },
  {
    id: 'fine-tuning',
    title: 'Model Fine-Tuning Pipeline',
    description: 'End-to-end fine-tuning pipeline for customizing open-source LLMs.',
    category: 'advanced',
    tags: ['python', 'pytorch', 'docker'],
    level: 'advanced',
    image: '/examples/fine-tuning.png',
    githubUrl: 'https://github.com/proksi/examples/fine-tuning-pipeline',
    demoUrl: null,
    features: [
      'Dataset preparation',
      'Training configuration',
      'Model quantization',
      'Evaluation metrics',
    ]
  },
  {
    id: 'semantic-search',
    title: 'Vector Search API',
    description: 'API for semantic search across multiple vector databases with unified interface.',
    category: 'integration',
    tags: ['typescript', 'express', 'vector-db'],
    level: 'intermediate',
    image: '/examples/semantic-search.png',
    githubUrl: 'https://github.com/proksi/examples/vector-search-api',
    demoUrl: 'https://vector-demo.proksi.dev',
    features: [
      'Multiple vector DB support',
      'Query optimization',
      'Metadata filtering',
      'Hybrid search',
    ]
  },
  {
    id: 'prompt-engineering',
    title: 'Prompt Engineering Studio',
    description: 'Create, test, and optimize prompts for different LLM providers.',
    category: 'tool',
    tags: ['react', 'javascript', 'prompt-engineering'],
    level: 'beginner',
    image: '/examples/prompt-studio.png',
    githubUrl: 'https://github.com/proksi/examples/prompt-studio',
    demoUrl: 'https://prompt-demo.proksi.dev',
    features: [
      'Visual prompt editor',
      'Template variables',
      'A/B testing',
      'Performance analytics',
    ]
  },
  {
    id: 'agent-framework',
    title: 'AI Agent Framework',
    description: 'Build autonomous AI agents with tools, memory, and planning capabilities.',
    category: 'advanced',
    tags: ['typescript', 'react', 'tools'],
    level: 'advanced',
    image: '/examples/agent-framework.png',
    githubUrl: 'https://github.com/proksi/examples/agent-framework',
    demoUrl: 'https://agent-demo.proksi.dev',
    features: [
      'Tool integration',
      'Long-term memory',
      'ReAct planning',
      'Multi-agent communication',
    ]
  },
]

// Starter templates data
const starterTemplates = [
  {
    id: 'nextjs-starter',
    title: 'Next.js Starter',
    description: 'Start a new project with Next.js, Tailwind, and Proksi SDK.',
    command: 'npx create-proksi-app my-app --template nextjs',
    language: 'typescript',
    framework: 'nextjs'
  },
  {
    id: 'express-starter',
    title: 'Express API Starter',
    description: 'Express.js API backend with Proksi SDK integration.',
    command: 'npx create-proksi-app my-api --template express',
    language: 'javascript',
    framework: 'express'
  },
  {
    id: 'fastapi-starter',
    title: 'FastAPI Python Starter',
    description: 'Python FastAPI backend with Proksi client library.',
    command: 'pip install proksi-cli && proksi init --template fastapi',
    language: 'python',
    framework: 'fastapi'
  },
  {
    id: 'react-starter',
    title: 'React Frontend Starter',
    description: 'React SPA with Proksi SDK integration.',
    command: 'npx create-proksi-app my-react-app --template react',
    language: 'typescript',
    framework: 'react'
  },
]

// Code snippets data
const codeSnippets = [
  {
    id: 'basic-completion',
    title: 'Basic Completion',
    language: 'javascript',
    code: `import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://api.your-proksi-instance.com',
  apiKey: process.env.PROKSI_API_KEY
});

async function getCompletion() {
  const response = await client.completion({
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'What is an AI gateway?' }
    ]
  });
  
  console.log(response.choices[0].message.content);
}

getCompletion();`
  },
  {
    id: 'streaming-completion',
    title: 'Streaming Completion',
    language: 'javascript',
    code: `import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://api.your-proksi-instance.com',
  apiKey: process.env.PROKSI_API_KEY
});

async function streamCompletion() {
  const stream = await client.streamCompletion({
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Write a short story about AI.' }
    ]
  });
  
  // In Node.js
  for await (const chunk of stream) {
    process.stdout.write(chunk.choices[0]?.delta?.content || '');
  }
  
  // In the browser
  /*
  for await (const chunk of stream) {
    const content = chunk.choices[0]?.delta?.content || '';
    // Append content to UI
  }
  */
}`
  },
  {
    id: 'vector-operations',
    title: 'Vector Database Operations',
    language: 'javascript',
    code: `import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://api.your-proksi-instance.com',
  apiKey: process.env.PROKSI_API_KEY
});

async function vectorDemo() {
  // Upsert vectors
  await client.upsertVectors({
    vectors: [
      { 
        id: 'doc1', 
        values: [0.1, 0.2, 0.3, ...], // Your embedding vector
        metadata: { source: 'article', title: 'AI Basics' } 
      }
    ]
  });
  
  // Search for similar vectors
  const results = await client.searchVectors({
    queryVector: [0.2, 0.3, 0.4, ...], // Your query vector
    topK: 5
  });
  
  console.log(results);
}`
  },
  {
    id: 'multi-provider',
    title: 'Multi-Provider Routing',
    language: 'javascript',
    code: `import { ProksiClient } from 'proksi-sdk';

const client = new ProksiClient({
  baseUrl: 'https://api.your-proksi-instance.com',
  apiKey: process.env.PROKSI_API_KEY
});

async function multiProviderDemo() {
  // Route to specific provider
  const openaiResponse = await client.completion({
    provider: 'openai', // Explicitly specify provider
    messages: [
      { role: 'user', content: 'What is quantum computing?' }
    ]
  });
  
  // Let Proksi decide based on routing rules
  const autoRoutedResponse = await client.completion({
    messages: [
      { role: 'user', content: 'What is quantum computing?' }
    ],
    routingStrategy: 'cost_optimized' // Other options: 'quality', 'speed'
  });
  
  console.log('OpenAI:', openaiResponse.choices[0].message.content);
  console.log('Auto-routed:', autoRoutedResponse.choices[0].message.content);
}`
  },
]

export default function ExamplesPage() {
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div>
          <h1 className="text-3xl font-bold">Examples & Templates</h1>
          <p className="text-muted-foreground">
            Reference examples, starter templates and code snippets for integrating with Proksi AI Gateway
          </p>
        </div>
        
        <Tabs defaultValue="examples" className="mt-6">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="examples">Example Projects</TabsTrigger>
            <TabsTrigger value="templates">Starter Templates</TabsTrigger>
            <TabsTrigger value="snippets">Code Snippets</TabsTrigger>
          </TabsList>
          
          <TabsContent value="examples" className="space-y-6">
            <div className="flex justify-between items-center">
              <div className="space-y-1">
                <h2 className="text-2xl font-semibold tracking-tight">
                  Example Projects
                </h2>
                <p className="text-sm text-muted-foreground">
                  Ready-to-use projects showcasing Proksi AI Gateway capabilities
                </p>
              </div>
              
              <div className="flex gap-2">
                <Input
                  placeholder="Search examples..."
                  className="w-[200px]"
                />
                <Button variant="outline" size="icon">
                  <Search className="h-4 w-4" />
                </Button>
              </div>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {exampleProjects.map((project) => (
                <Card key={project.id} className="overflow-hidden flex flex-col">
                  <div className="bg-muted h-48 flex items-center justify-center">
                    {/* Placeholder for project image */}
                    <div className="flex items-center justify-center bg-primary/10 w-full h-full">
                      {project.category === 'application' && <MessagesSquare className="h-16 w-16 text-primary/60" />}
                      {project.category === 'integration' && <GitBranch className="h-16 w-16 text-primary/60" />}
                      {project.category === 'advanced' && <Cpu className="h-16 w-16 text-primary/60" />}
                      {project.category === 'tool' && <Bot className="h-16 w-16 text-primary/60" />}
                    </div>
                  </div>
                  
                  <CardHeader className="pb-2">
                    <div className="flex justify-between items-start">
                      <CardTitle className="text-xl">{project.title}</CardTitle>
                      <Badge variant={
                        project.level === 'beginner' ? 'default' : 
                        project.level === 'intermediate' ? 'secondary' : 
                        'outline'
                      }>
                        {project.level}
                      </Badge>
                    </div>
                    <CardDescription>{project.description}</CardDescription>
                  </CardHeader>
                  
                  <CardContent className="pb-0 flex-grow">
                    <div className="flex flex-wrap gap-1 mb-4">
                      {project.tags.map(tag => (
                        <Badge key={tag} variant="outline">{tag}</Badge>
                      ))}
                    </div>
                    
                    <div className="space-y-1">
                      <h4 className="text-sm font-medium">Features:</h4>
                      <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
                        {project.features.map((feature, i) => (
                          <li key={i}>{feature}</li>
                        ))}
                      </ul>
                    </div>
                  </CardContent>
                  
                  <CardFooter className="pt-4 flex gap-2">
                    <Button className="flex-1" asChild>
                      <a href={project.githubUrl} target="_blank" rel="noopener noreferrer">
                        <FolderGit2 className="mr-2 h-4 w-4" />
                        View Code
                      </a>
                    </Button>
                    {project.demoUrl && (
                      <Button variant="outline" className="flex-1" asChild>
                        <a href={project.demoUrl} target="_blank" rel="noopener noreferrer">
                          Live Demo
                        </a>
                      </Button>
                    )}
                  </CardFooter>
                </Card>
              ))}
            </div>
          </TabsContent>
          
          <TabsContent value="templates" className="space-y-6">
            <div className="space-y-1">
              <h2 className="text-2xl font-semibold tracking-tight">
                Starter Templates
              </h2>
              <p className="text-sm text-muted-foreground">
                Get started quickly with these project templates
              </p>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {starterTemplates.map((template) => (
                <Card key={template.id} className="flex flex-col">
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div>
                        <CardTitle>{template.title}</CardTitle>
                        <CardDescription className="mt-1">{template.description}</CardDescription>
                      </div>
                      <div className="flex gap-1">
                        <Badge>{template.language}</Badge>
                        <Badge variant="outline">{template.framework}</Badge>
                      </div>
                    </div>
                  </CardHeader>
                  
                  <CardContent className="flex-grow pb-0">
                    <div className="rounded-md bg-muted p-4 font-mono text-sm overflow-x-auto">
                      <code>{template.command}</code>
                    </div>
                  </CardContent>
                  
                  <CardFooter className="pt-4">
                    <Button variant="outline" className="w-full">
                      <FileCode className="mr-2 h-4 w-4" />
                      View Documentation
                    </Button>
                  </CardFooter>
                </Card>
              ))}
            </div>
          </TabsContent>
          
          <TabsContent value="snippets" className="space-y-6">
            <div className="space-y-1">
              <h2 className="text-2xl font-semibold tracking-tight">
                Code Snippets
              </h2>
              <p className="text-sm text-muted-foreground">
                Ready-to-use code examples for common scenarios
              </p>
            </div>
            
            <div className="space-y-6">
              {codeSnippets.map((snippet) => (
                <Card key={snippet.id}>
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-lg">{snippet.title}</CardTitle>
                      <Badge variant="outline">{snippet.language}</Badge>
                    </div>
                  </CardHeader>
                  
                  <CardContent>
                    <div className="rounded-md bg-muted p-4 font-mono text-sm overflow-x-auto">
                      <pre><code>{snippet.code}</code></pre>
                    </div>
                  </CardContent>
                  
                  <CardFooter className="flex justify-end">
                    <Button variant="ghost" size="sm" onClick={() => {
                      navigator.clipboard.writeText(snippet.code);
                    }}>
                      <Code className="mr-2 h-4 w-4" />
                      Copy Code
                    </Button>
                  </CardFooter>
                </Card>
              ))}
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
} 