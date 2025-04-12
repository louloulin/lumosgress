"use client"

import { useState } from "react"
import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Switch } from "@/components/ui/switch"
import { PlusIcon, PencilIcon, TrashIcon } from "@/components/icons/icons"

// Define the plugin type
interface Plugin {
  id: string
  name: string
  description: string
  version: string
  status: 'active' | 'inactive' | 'error'
  type: 'llm' | 'vector' | 'security' | 'analytics'
  config: Record<string, any>
}

export default function PluginsPage() {
  const [activeTab, setActiveTab] = useState("all")
  
  // Mock data - would come from API in real implementation
  const plugins: Plugin[] = [
    {
      id: "1",
      name: "LLM Router",
      description: "Routes requests to different LLM providers based on content",
      version: "1.0.0",
      status: "active",
      type: "llm",
      config: {
        defaultProvider: "openai",
        fallbackProvider: "anthropic"
      }
    },
    {
      id: "2",
      name: "Vector Database",
      description: "Integrates with vector databases for semantic search",
      version: "1.0.0",
      status: "active",
      type: "vector",
      config: {
        provider: "pinecone",
        index: "default"
      }
    },
    {
      id: "3",
      name: "Prompt Debugger",
      description: "Analyzes and optimizes prompt quality",
      version: "1.0.0",
      status: "active",
      type: "llm",
      config: {
        rules: ["length", "clarity", "safety"]
      }
    },
    {
      id: "4",
      name: "Anomaly Detection",
      description: "Detects unusual patterns in AI traffic",
      version: "1.0.0",
      status: "active",
      type: "analytics",
      config: {
        algorithms: ["zscore", "moving_average"]
      }
    }
  ]
  
  // Filter plugins based on active tab
  const filteredPlugins = activeTab === "all" 
    ? plugins 
    : plugins.filter(plugin => plugin.type === activeTab)
  
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Plugins Management</h1>
            <p className="text-muted-foreground">
              Configure and manage your AI Gateway plugins
            </p>
          </div>
          <Button>
            <PlusIcon className="mr-2 h-4 w-4" />
            Install Plugin
          </Button>
        </div>
        
        <Card className="mt-6">
          <CardHeader>
            <CardTitle>Plugins</CardTitle>
            <CardDescription>
              Manage and configure your AI Gateway plugins
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center space-x-2">
                <Input 
                  placeholder="Search plugins..." 
                  className="max-w-sm" 
                />
                <Select defaultValue="all">
                  <SelectTrigger className="w-[180px]">
                    <SelectValue placeholder="Status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Statuses</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="inactive">Inactive</SelectItem>
                    <SelectItem value="error">Error</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <Tabs defaultValue="all" className="w-full" onValueChange={setActiveTab}>
                <TabsList>
                  <TabsTrigger value="all">All Plugins</TabsTrigger>
                  <TabsTrigger value="llm">LLM Plugins</TabsTrigger>
                  <TabsTrigger value="vector">Vector Plugins</TabsTrigger>
                  <TabsTrigger value="security">Security Plugins</TabsTrigger>
                  <TabsTrigger value="analytics">Analytics Plugins</TabsTrigger>
                </TabsList>
                
                <TabsContent value="all" className="mt-4">
                  <PluginsList plugins={filteredPlugins} />
                </TabsContent>
                <TabsContent value="llm" className="mt-4">
                  <PluginsList plugins={filteredPlugins} />
                </TabsContent>
                <TabsContent value="vector" className="mt-4">
                  <PluginsList plugins={filteredPlugins} />
                </TabsContent>
                <TabsContent value="security" className="mt-4">
                  <PluginsList plugins={filteredPlugins} />
                </TabsContent>
                <TabsContent value="analytics" className="mt-4">
                  <PluginsList plugins={filteredPlugins} />
                </TabsContent>
              </Tabs>
            </div>
          </CardContent>
        </Card>
        
        <Card className="mt-4">
          <CardHeader>
            <CardTitle>Plugin Configuration</CardTitle>
            <CardDescription>
              Configure plugin settings and parameters
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="plugin-name">Plugin Name</Label>
                  <Input id="plugin-name" placeholder="LLM Router" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="plugin-version">Version</Label>
                  <Input id="plugin-version" placeholder="1.0.0" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="plugin-type">Type</Label>
                  <Select defaultValue="llm">
                    <SelectTrigger id="plugin-type">
                      <SelectValue placeholder="Select type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="llm">LLM</SelectItem>
                      <SelectItem value="vector">Vector</SelectItem>
                      <SelectItem value="security">Security</SelectItem>
                      <SelectItem value="analytics">Analytics</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-center space-x-2 pt-8">
                  <Switch id="plugin-active" defaultChecked />
                  <Label htmlFor="plugin-active">Active</Label>
                </div>
              </div>
              
              <div className="pt-4 flex justify-end space-x-2">
                <Button variant="outline">Cancel</Button>
                <Button>Save Configuration</Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  )
}

// Plugins list component
function PluginsList({ plugins }: { plugins: Plugin[] }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>Description</TableHead>
          <TableHead>Version</TableHead>
          <TableHead>Type</TableHead>
          <TableHead>Status</TableHead>
          <TableHead className="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {plugins.map(plugin => (
          <TableRow key={plugin.id}>
            <TableCell className="font-medium">{plugin.name}</TableCell>
            <TableCell>{plugin.description}</TableCell>
            <TableCell>{plugin.version}</TableCell>
            <TableCell>
              <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium
                ${plugin.type === 'llm' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300' : 
                  plugin.type === 'vector' ? 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300' : 
                  plugin.type === 'security' ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300' :
                  'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300'}`}>
                {plugin.type}
              </span>
            </TableCell>
            <TableCell>
              <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                plugin.status === 'active' 
                  ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300' 
                  : plugin.status === 'error'
                  ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
                  : 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300'
              }`}>
                {plugin.status}
              </span>
            </TableCell>
            <TableCell className="text-right">
              <div className="flex justify-end space-x-1">
                <Button variant="ghost" size="icon">
                  <PencilIcon className="h-4 w-4" />
                  <span className="sr-only">Edit</span>
                </Button>
                <Button variant="ghost" size="icon">
                  <TrashIcon className="h-4 w-4" />
                  <span className="sr-only">Delete</span>
                </Button>
              </div>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
} 