"use client"

import { useState } from "react"
import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { AlertCircle, Check, Copy, Save } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

export default function ApiConfigPage() {
  const [activeTab, setActiveTab] = useState("endpoints")
  const [testResponse, setTestResponse] = useState<{success: boolean, message: string} | null>(null)
  
  const handleTestConnection = () => {
    // Simulate API test
    setTimeout(() => {
      setTestResponse({
        success: true,
        message: "Successfully connected to the API endpoint."
      })
    }, 1000)
  }
  
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div>
          <h1 className="text-3xl font-bold">API Configuration</h1>
          <p className="text-muted-foreground">
            Manage API endpoints, keys, and test API functionality
          </p>
        </div>
        
        <Tabs defaultValue="endpoints" className="w-full mt-6" onValueChange={setActiveTab}>
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="endpoints">Endpoints</TabsTrigger>
            <TabsTrigger value="keys">API Keys</TabsTrigger>
            <TabsTrigger value="test">Test API</TabsTrigger>
          </TabsList>
          
          <TabsContent value="endpoints" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle>Configure API Endpoints</CardTitle>
                <CardDescription>
                  Customize the endpoints for different providers and services
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-6">
                  <div className="grid gap-3">
                    <Label htmlFor="openaiEndpoint">OpenAI Endpoint</Label>
                    <Input 
                      id="openaiEndpoint" 
                      placeholder="https://api.openai.com/v1" 
                      defaultValue="https://api.openai.com/v1"
                    />
                  </div>
                  
                  <div className="grid gap-3">
                    <Label htmlFor="anthropicEndpoint">Anthropic Endpoint</Label>
                    <Input 
                      id="anthropicEndpoint" 
                      placeholder="https://api.anthropic.com" 
                      defaultValue="https://api.anthropic.com"
                    />
                  </div>
                  
                  <div className="grid gap-3">
                    <Label htmlFor="customEndpoint">Custom Endpoint</Label>
                    <Input 
                      id="customEndpoint" 
                      placeholder="https://your-custom-endpoint.com" 
                    />
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Switch id="useProxy" />
                    <Label htmlFor="useProxy">Use API Proxy</Label>
                  </div>
                </div>
              </CardContent>
              <CardFooter className="flex justify-between">
                <Button variant="outline">Reset to Defaults</Button>
                <Button>Save Changes</Button>
              </CardFooter>
            </Card>
          </TabsContent>
          
          <TabsContent value="keys" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle>API Keys Management</CardTitle>
                <CardDescription>
                  Add and manage API keys for different providers
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-6">
                  <div className="grid gap-3">
                    <Label htmlFor="openaiKey">OpenAI API Key</Label>
                    <div className="flex gap-2">
                      <Input 
                        id="openaiKey" 
                        type="password" 
                        placeholder="sk-..." 
                        className="flex-1"
                      />
                      <Button variant="outline" size="icon">
                        <Copy className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                  
                  <div className="grid gap-3">
                    <Label htmlFor="anthropicKey">Anthropic API Key</Label>
                    <div className="flex gap-2">
                      <Input 
                        id="anthropicKey" 
                        type="password" 
                        placeholder="sk-ant-..." 
                        className="flex-1"
                      />
                      <Button variant="outline" size="icon">
                        <Copy className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                  
                  <div className="grid gap-3">
                    <Label htmlFor="customKey">Custom API Key</Label>
                    <div className="flex gap-2">
                      <Input 
                        id="customKey" 
                        type="password" 
                        placeholder="Custom API key" 
                        className="flex-1"
                      />
                      <Button variant="outline" size="icon">
                        <Copy className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              </CardContent>
              <CardFooter className="flex justify-end">
                <Button>
                  <Save className="mr-2 h-4 w-4" />
                  Save Keys
                </Button>
              </CardFooter>
            </Card>
          </TabsContent>
          
          <TabsContent value="test" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle>Test API Connectivity</CardTitle>
                <CardDescription>
                  Verify your API endpoints and keys are working correctly
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-6">
                  <div className="grid gap-3">
                    <Label htmlFor="testProvider">Select Provider</Label>
                    <Select defaultValue="openai">
                      <SelectTrigger>
                        <SelectValue placeholder="Select provider" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="openai">OpenAI</SelectItem>
                        <SelectItem value="anthropic">Anthropic</SelectItem>
                        <SelectItem value="custom">Custom</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div className="grid gap-3">
                    <Label htmlFor="testRequest">Test Request</Label>
                    <Textarea 
                      id="testRequest" 
                      placeholder="Enter your test request here"
                      defaultValue='{"messages": [{"role": "user", "content": "Hello, world!"}]}'
                      rows={5}
                    />
                  </div>
                  
                  {testResponse && (
                    <Alert variant={testResponse.success ? "default" : "destructive"}>
                      {testResponse.success ? (
                        <Check className="h-4 w-4" />
                      ) : (
                        <AlertCircle className="h-4 w-4" />
                      )}
                      <AlertTitle>
                        {testResponse.success ? "Success" : "Error"}
                      </AlertTitle>
                      <AlertDescription>
                        {testResponse.message}
                      </AlertDescription>
                    </Alert>
                  )}
                </div>
              </CardContent>
              <CardFooter className="flex justify-end">
                <Button onClick={handleTestConnection}>
                  Test Connection
                </Button>
              </CardFooter>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
} 