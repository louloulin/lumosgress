"use client"

import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Switch } from "@/components/ui/switch"
import { useState } from "react"

// Mock data for plugins
const mockPlugins = [
  {
    id: "1",
    name: "Prompt Transformer",
    description: "Transforms and enhances prompts before they reach LLM providers",
    status: true,
    version: "1.2.0",
    config: {
      system_message: "You are a helpful AI assistant.",
      context_length: 4096,
      safety_filters: true
    }
  },
  {
    id: "2",
    name: "LLM Router",
    description: "Routes requests to appropriate LLM providers based on content and rules",
    status: true,
    version: "1.1.5",
    config: {
      default_provider: "openai",
      fallback_provider: "anthropic",
      routing_strategy: "content_based"
    }
  },
  {
    id: "3",
    name: "AI Security",
    description: "Secures AI interactions by detecting and preventing prompt injections",
    status: true,
    version: "1.0.2",
    config: {
      injection_detection: true,
      pii_detection: true,
      content_filtering: "medium"
    }
  },
  {
    id: "4",
    name: "Vector Database",
    description: "Integrates with vector databases for similarity search and embeddings",
    status: true,
    version: "1.3.0",
    config: {
      provider: "pinecone",
      dimensions: 1536,
      metric: "cosine"
    }
  },
  {
    id: "5",
    name: "Model Aggregator",
    description: "Aggregates responses from multiple models into a single coherent response",
    status: true,
    version: "0.9.1",
    config: {
      strategy: "weighted",
      models: ["gpt-4", "claude-2"],
      weights: [0.7, 0.3]
    }
  },
  {
    id: "6",
    name: "AI Analytics",
    description: "Collects and analyzes AI usage metrics and performance data",
    status: false,
    version: "0.8.5",
    config: {
      storage: "postgres",
      retention_days: 90,
      metrics_interval: "1m"
    }
  },
  {
    id: "7",
    name: "Prompt Debugger",
    description: "Helps analyze and debug prompts to improve quality and effectiveness",
    status: false,
    version: "0.7.0",
    config: {
      analysis_rules: ["clarity", "specificity", "length"],
      suggestions: true
    }
  }
]

export default function PluginsPage() {
  const [plugins, setPlugins] = useState(mockPlugins)

  const togglePluginStatus = (id: string) => {
    setPlugins(
      plugins.map((plugin) =>
        plugin.id === id ? { ...plugin, status: !plugin.status } : plugin
      )
    )
  }

  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Plugins</h1>
            <p className="text-muted-foreground">
              Manage and configure AI gateway plugins
            </p>
          </div>
          <Button>
            Add Plugin
          </Button>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Installed Plugins</CardTitle>
            <CardDescription>
              View and manage all installed plugins
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Version</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {plugins.map((plugin) => (
                  <TableRow key={plugin.id}>
                    <TableCell className="font-medium">{plugin.name}</TableCell>
                    <TableCell>{plugin.description}</TableCell>
                    <TableCell>{plugin.version}</TableCell>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        <Switch
                          checked={plugin.status}
                          onCheckedChange={() => togglePluginStatus(plugin.id)}
                        />
                        <span>{plugin.status ? "Active" : "Inactive"}</span>
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button variant="outline" size="sm">
                        Configure
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
          <CardFooter className="flex justify-between">
            <Button variant="outline">Refresh</Button>
            <Button variant="outline">Export Configuration</Button>
          </CardFooter>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Plugin Marketplace</CardTitle>
            <CardDescription>
              Discover and install new plugins
            </CardDescription>
          </CardHeader>
          <CardContent className="h-[200px] flex items-center justify-center">
            <div className="text-center text-muted-foreground">
              <p>Plugin marketplace is coming soon!</p>
              <p className="text-sm">Browse and install community plugins with one click</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  )
} 