"use client"

import { useState } from "react"
import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"

// Define provider types
interface ModelConfig {
  id: string
  enabled: boolean
  contextWindow: number
  costPer1kTokens: number
}

interface BaseProviderConfig {
  name: string
  enabled: boolean
  apiKey: string
  baseUrl: string
  defaultModel: string
  availableModels: string[]
  models: ModelConfig[]
}

interface AzureProviderConfig extends BaseProviderConfig {
  resourceName: string
  deploymentId: string
  apiVersion: string
}

type ProviderConfig = BaseProviderConfig | AzureProviderConfig

interface Providers {
  openai: BaseProviderConfig
  anthropic: BaseProviderConfig
  google: BaseProviderConfig
  azure: AzureProviderConfig
}

// Mock provider data
const mockProviders: Providers = {
  openai: {
    name: "OpenAI",
    enabled: true,
    apiKey: "sk-*****************************",
    baseUrl: "https://api.openai.com/v1",
    defaultModel: "gpt-4",
    availableModels: ["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"],
    models: [
      {
        id: "gpt-3.5-turbo",
        enabled: true,
        contextWindow: 16385,
        costPer1kTokens: 0.0015
      },
      {
        id: "gpt-4",
        enabled: true,
        contextWindow: 8192,
        costPer1kTokens: 0.03
      },
      {
        id: "gpt-4-turbo",
        enabled: true,
        contextWindow: 128000,
        costPer1kTokens: 0.01
      }
    ]
  },
  anthropic: {
    name: "Anthropic",
    enabled: true,
    apiKey: "sk-ant-*****************************",
    baseUrl: "https://api.anthropic.com",
    defaultModel: "claude-3-opus-20240229",
    availableModels: ["claude-2.1", "claude-3-opus-20240229", "claude-3-sonnet-20240229", "claude-3-haiku-20240307"],
    models: [
      {
        id: "claude-2.1",
        enabled: true,
        contextWindow: 100000,
        costPer1kTokens: 0.008
      },
      {
        id: "claude-3-opus-20240229",
        enabled: true,
        contextWindow: 200000,
        costPer1kTokens: 0.015
      },
      {
        id: "claude-3-sonnet-20240229",
        enabled: true,
        contextWindow: 200000,
        costPer1kTokens: 0.003
      },
      {
        id: "claude-3-haiku-20240307",
        enabled: true,
        contextWindow: 200000,
        costPer1kTokens: 0.00025
      }
    ]
  },
  google: {
    name: "Google AI (Gemini)",
    enabled: false,
    apiKey: "",
    baseUrl: "https://generativelanguage.googleapis.com",
    defaultModel: "gemini-pro",
    availableModels: ["gemini-pro", "gemini-ultra"],
    models: [
      {
        id: "gemini-pro",
        enabled: true,
        contextWindow: 32768,
        costPer1kTokens: 0.0005
      },
      {
        id: "gemini-ultra",
        enabled: true,
        contextWindow: 32768,
        costPer1kTokens: 0.0005
      }
    ]
  },
  azure: {
    name: "Azure OpenAI",
    enabled: false,
    apiKey: "",
    baseUrl: "",
    resourceName: "",
    defaultModel: "gpt-4",
    deploymentId: "",
    apiVersion: "2024-05-01",
    availableModels: ["gpt-35-turbo", "gpt-4", "gpt-4-turbo"],
    models: [
      {
        id: "gpt-35-turbo",
        enabled: true,
        contextWindow: 16385,
        costPer1kTokens: 0.0015
      },
      {
        id: "gpt-4",
        enabled: true,
        contextWindow: 8192,
        costPer1kTokens: 0.03
      },
      {
        id: "gpt-4-turbo",
        enabled: true,
        contextWindow: 128000,
        costPer1kTokens: 0.01
      }
    ]
  }
}

export default function LLMProvidersPage() {
  const [providers, setProviders] = useState<Providers>(mockProviders)
  const [activeTab, setActiveTab] = useState("openai")

  const toggleProviderStatus = (providerId: keyof Providers) => {
    setProviders({
      ...providers,
      [providerId]: {
        ...providers[providerId],
        enabled: !providers[providerId].enabled
      }
    })
  }

  const updateApiKey = (providerId: keyof Providers, value: string) => {
    setProviders({
      ...providers,
      [providerId]: {
        ...providers[providerId],
        apiKey: value
      }
    })
  }

  const updateDefaultModel = (providerId: keyof Providers, value: string) => {
    setProviders({
      ...providers,
      [providerId]: {
        ...providers[providerId],
        defaultModel: value
      }
    })
  }

  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">LLM Providers</h1>
            <p className="text-muted-foreground">
              Configure and manage your LLM provider integrations
            </p>
          </div>
          <Dialog>
            <DialogTrigger asChild>
              <Button>Add Provider</Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
              <DialogHeader>
                <DialogTitle>Add LLM Provider</DialogTitle>
                <DialogDescription>
                  Configure a new LLM provider to use with the AI gateway.
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid gap-2">
                  <Label htmlFor="provider-type">Provider Type</Label>
                  <Select defaultValue="custom">
                    <SelectTrigger id="provider-type">
                      <SelectValue placeholder="Select provider type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="openai">OpenAI</SelectItem>
                      <SelectItem value="anthropic">Anthropic</SelectItem>
                      <SelectItem value="google">Google AI (Gemini)</SelectItem>
                      <SelectItem value="azure">Azure OpenAI</SelectItem>
                      <SelectItem value="custom">Custom Provider</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="provider-name">Provider Name</Label>
                  <Input id="provider-name" placeholder="Enter provider name" />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="api-key">API Key</Label>
                  <Input id="api-key" type="password" placeholder="Enter API key" />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="base-url">Base URL</Label>
                  <Input id="base-url" placeholder="https://api.example.com" />
                </div>
              </div>
              <DialogFooter>
                <Button type="submit">Save Provider</Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>

        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="openai">OpenAI</TabsTrigger>
            <TabsTrigger value="anthropic">Anthropic</TabsTrigger>
            <TabsTrigger value="google">Google AI</TabsTrigger>
            <TabsTrigger value="azure">Azure OpenAI</TabsTrigger>
          </TabsList>
          
          {Object.entries(providers).map(([id, provider]) => (
            <TabsContent key={id} value={id} className="space-y-4">
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>{provider.name} Configuration</CardTitle>
                      <CardDescription>
                        Configure your {provider.name} integration settings
                      </CardDescription>
                    </div>
                    <div className="flex items-center gap-2">
                      <Switch 
                        checked={provider.enabled} 
                        onCheckedChange={() => toggleProviderStatus(id as keyof Providers)}
                      />
                      <span>{provider.enabled ? "Enabled" : "Disabled"}</span>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid gap-2">
                    <Label htmlFor={`${id}-api-key`}>API Key</Label>
                    <Input 
                      id={`${id}-api-key`} 
                      type="password" 
                      value={provider.apiKey} 
                      onChange={(e) => updateApiKey(id as keyof Providers, e.target.value)}
                      placeholder="Enter your API key"
                    />
                  </div>
                  
                  <div className="grid gap-2">
                    <Label htmlFor={`${id}-base-url`}>Base URL</Label>
                    <Input 
                      id={`${id}-base-url`} 
                      value={provider.baseUrl} 
                      placeholder="https://api.example.com"
                    />
                  </div>
                  
                  {id === 'azure' && (
                    <>
                      <div className="grid gap-2">
                        <Label htmlFor="azure-resource">Resource Name</Label>
                        <Input 
                          id="azure-resource" 
                          value={(provider as AzureProviderConfig).resourceName} 
                          placeholder="your-resource-name"
                        />
                      </div>
                      <div className="grid gap-2">
                        <Label htmlFor="azure-deployment">Deployment ID</Label>
                        <Input 
                          id="azure-deployment" 
                          value={(provider as AzureProviderConfig).deploymentId} 
                          placeholder="deployment-id"
                        />
                      </div>
                      <div className="grid gap-2">
                        <Label htmlFor="azure-api-version">API Version</Label>
                        <Input 
                          id="azure-api-version" 
                          value={(provider as AzureProviderConfig).apiVersion} 
                          placeholder="2024-05-01"
                        />
                      </div>
                    </>
                  )}
                  
                  <div className="grid gap-2">
                    <Label htmlFor={`${id}-default-model`}>Default Model</Label>
                    <Select 
                      value={provider.defaultModel}
                      onValueChange={(value) => updateDefaultModel(id as keyof Providers, value)}
                    >
                      <SelectTrigger id={`${id}-default-model`}>
                        <SelectValue placeholder="Select default model" />
                      </SelectTrigger>
                      <SelectContent>
                        {provider.availableModels.map((model: string) => (
                          <SelectItem key={model} value={model}>
                            {model}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </CardContent>
                <CardFooter className="flex justify-between">
                  <Button variant="outline">Test Connection</Button>
                  <Button>Save Changes</Button>
                </CardFooter>
              </Card>
              
              <Card>
                <CardHeader>
                  <CardTitle>Available Models</CardTitle>
                  <CardDescription>
                    Configure individual model settings
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-4">
                    {provider.models.map((model: ModelConfig) => (
                      <div key={model.id} className="flex items-center justify-between rounded-lg border p-4">
                        <div>
                          <h3 className="font-medium">{model.id}</h3>
                          <div className="mt-1 text-sm text-muted-foreground">
                            Context: {model.contextWindow.toLocaleString()} tokens | 
                            Cost: ${model.costPer1kTokens} per 1k tokens
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <Switch checked={model.enabled} />
                          <span>{model.enabled ? "Enabled" : "Disabled"}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          ))}
        </Tabs>
      </div>
    </DashboardLayout>
  )
} 