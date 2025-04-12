'use client'

import { useState } from 'react'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import { PlusIcon } from '@/components/icons/icons'

interface RequestTemplate {
  name: string
  endpoint: string
  method: string
  headers: Record<string, string>
  body: string
}

const defaultTemplates: RequestTemplate[] = [
  {
    name: 'OpenAI Chat',
    endpoint: 'https://api.openai.com/v1/chat/completions',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer YOUR_API_KEY'
    },
    body: JSON.stringify({
      model: 'gpt-4',
      messages: [
        {
          role: 'user',
          content: 'Hello, how are you?'
        }
      ],
      temperature: 0.7
    }, null, 2)
  },
  {
    name: 'Anthropic Claude',
    endpoint: 'https://api.anthropic.com/v1/messages',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': 'YOUR_API_KEY',
      'anthropic-version': '2023-06-01'
    },
    body: JSON.stringify({
      model: 'claude-3-opus-20240229',
      max_tokens: 1024,
      messages: [
        {
          role: 'user',
          content: 'Hello, how are you?'
        }
      ]
    }, null, 2)
  }
]

export default function RequestBuilderPage() {
  const [selectedTemplate, setSelectedTemplate] = useState<RequestTemplate>(defaultTemplates[0])
  const [response, setResponse] = useState<string>('')
  const [isLoading, setIsLoading] = useState(false)

  const handleSendRequest = async () => {
    setIsLoading(true)
    try {
      const response = await fetch('/api/ai-request-builder', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          template: selectedTemplate,
          body: selectedTemplate.body,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Failed to send request')
      }

      const data = await response.json()
      setResponse(JSON.stringify(data, null, 2))
    } catch (error) {
      if (error instanceof Error) {
        setResponse(`Error: ${error.message}`)
      } else {
        setResponse('An unknown error occurred')
      }
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="container mx-auto py-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">AI Request Builder</h1>
        <Button>
          <PlusIcon className="w-4 h-4 mr-2" />
          New Template
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="p-6">
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium mb-2 block">Template</label>
              <Select
                value={selectedTemplate.name}
                onValueChange={(value) => {
                  const template = defaultTemplates.find(t => t.name === value)
                  if (template) setSelectedTemplate(template)
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a template" />
                </SelectTrigger>
                <SelectContent>
                  {defaultTemplates.map((template) => (
                    <SelectItem key={template.name} value={template.name}>
                      {template.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Endpoint</label>
              <Input value={selectedTemplate.endpoint} readOnly />
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Method</label>
              <Input value={selectedTemplate.method} readOnly />
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Headers</label>
              <Textarea
                value={JSON.stringify(selectedTemplate.headers, null, 2)}
                readOnly
                className="font-mono"
                rows={4}
              />
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Body</label>
              <Textarea
                value={selectedTemplate.body}
                readOnly
                className="font-mono"
                rows={8}
              />
            </div>

            <Button onClick={handleSendRequest} disabled={isLoading}>
              {isLoading ? 'Sending...' : 'Send Request'}
            </Button>
          </div>
        </Card>

        <Card className="p-6">
          <div className="space-y-4">
            <h2 className="text-lg font-medium">Response</h2>
            <Textarea
              value={response}
              readOnly
              className="font-mono"
              rows={20}
            />
          </div>
        </Card>
      </div>
    </div>
  )
} 