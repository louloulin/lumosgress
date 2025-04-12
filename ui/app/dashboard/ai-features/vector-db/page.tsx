"use client"

import { useState } from "react"
import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"

// Define vector database types
interface VectorDBConfig {
  name: string
  enabled: boolean
  type: "pinecone" | "qdrant" | "weaviate" | "milvus"
  apiKey?: string
  endpoint?: string
  namespace?: string
  dimensions: number
  metric: "cosine" | "euclidean" | "dot"
  indexName?: string
}

// Mock vector database configurations
const mockVectorDBs: VectorDBConfig[] = [
  {
    name: "Product Knowledge Base",
    enabled: true,
    type: "pinecone",
    apiKey: "**************************",
    endpoint: "https://product-kb-12345.svc.us-west1-gcp.pinecone.io",
    namespace: "products",
    dimensions: 1536,
    metric: "cosine",
    indexName: "product-kb"
  },
  {
    name: "Customer Support FAQ",
    enabled: true,
    type: "qdrant",
    endpoint: "https://qdrant-instance.company.com:6333",
    namespace: "support-faq",
    dimensions: 768,
    metric: "cosine",
    indexName: "support-collection"
  },
  {
    name: "Legal Documents",
    enabled: false,
    type: "weaviate",
    apiKey: "",
    endpoint: "",
    namespace: "legal",
    dimensions: 1536,
    metric: "cosine",
    indexName: "legal-docs"
  }
]

export default function VectorDBPage() {
  const [vectorDBs, setVectorDBs] = useState<VectorDBConfig[]>(mockVectorDBs)
  const [activeTab, setActiveTab] = useState("connections")

  const toggleDBStatus = (index: number) => {
    const updatedDBs = [...vectorDBs]
    updatedDBs[index].enabled = !updatedDBs[index].enabled
    setVectorDBs(updatedDBs)
  }

  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Vector Databases</h1>
            <p className="text-muted-foreground">
              Configure and manage vector database connections
            </p>
          </div>
          <Dialog>
            <DialogTrigger asChild>
              <Button>Add Connection</Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[550px]">
              <DialogHeader>
                <DialogTitle>Add Vector Database Connection</DialogTitle>
                <DialogDescription>
                  Configure a new vector database connection for RAG workflows.
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="grid gap-2">
                    <Label htmlFor="connection-name">Connection Name</Label>
                    <Input id="connection-name" placeholder="My Vector DB" />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="db-type">Database Type</Label>
                    <Select defaultValue="pinecone">
                      <SelectTrigger id="db-type">
                        <SelectValue placeholder="Select type" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="pinecone">Pinecone</SelectItem>
                        <SelectItem value="qdrant">Qdrant</SelectItem>
                        <SelectItem value="weaviate">Weaviate</SelectItem>
                        <SelectItem value="milvus">Milvus</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="api-key">API Key</Label>
                  <Input id="api-key" type="password" placeholder="Enter API key" />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="endpoint">Endpoint URL</Label>
                  <Input id="endpoint" placeholder="https://example.com" />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="grid gap-2">
                    <Label htmlFor="index-name">Index/Collection Name</Label>
                    <Input id="index-name" placeholder="my-index" />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="namespace">Namespace</Label>
                    <Input id="namespace" placeholder="default" />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="grid gap-2">
                    <Label htmlFor="dimensions">Dimensions</Label>
                    <Input id="dimensions" type="number" defaultValue="1536" />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="metric">Distance Metric</Label>
                    <Select defaultValue="cosine">
                      <SelectTrigger id="metric">
                        <SelectValue placeholder="Select metric" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="cosine">Cosine</SelectItem>
                        <SelectItem value="euclidean">Euclidean</SelectItem>
                        <SelectItem value="dot">Dot Product</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </div>
              <DialogFooter>
                <Button type="submit">Save Connection</Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>

        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            <TabsTrigger value="connections">Connections</TabsTrigger>
            <TabsTrigger value="operations">Operations</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>
          <TabsContent value="connections" className="space-y-4">
            {vectorDBs.map((db, index) => (
              <Card key={index}>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>{db.name}</CardTitle>
                      <CardDescription>
                        {db.type.charAt(0).toUpperCase() + db.type.slice(1)} • {db.dimensions} dimensions • {db.metric} similarity
                      </CardDescription>
                    </div>
                    <div className="flex items-center gap-2">
                      <Switch 
                        checked={db.enabled} 
                        onCheckedChange={() => toggleDBStatus(index)}
                      />
                      <span>{db.enabled ? "Active" : "Inactive"}</span>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div>
                      <div className="text-sm font-medium">Endpoint</div>
                      <div className="text-sm text-muted-foreground truncate">{db.endpoint || "Not configured"}</div>
                    </div>
                    <div>
                      <div className="text-sm font-medium">Index/Collection</div>
                      <div className="text-sm text-muted-foreground">{db.indexName || "Default"}</div>
                    </div>
                    <div>
                      <div className="text-sm font-medium">Namespace</div>
                      <div className="text-sm text-muted-foreground">{db.namespace || "Default"}</div>
                    </div>
                    <div>
                      <div className="text-sm font-medium">API Key</div>
                      <div className="text-sm text-muted-foreground">{db.apiKey ? "••••••••••••••••" : "Not configured"}</div>
                    </div>
                  </div>
                </CardContent>
                <CardFooter className="flex justify-between">
                  <Button variant="outline">Test Connection</Button>
                  <div className="flex gap-2">
                    <Button variant="outline">Edit</Button>
                    <Button variant="destructive">Delete</Button>
                  </div>
                </CardFooter>
              </Card>
            ))}
          </TabsContent>
          <TabsContent value="operations" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Vector Operations</CardTitle>
                <CardDescription>
                  Manage vector data operations
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="rounded-md border p-4">
                    <h3 className="font-medium">Upsert Vectors</h3>
                    <p className="mt-1 text-sm text-muted-foreground">
                      Add or update vectors in your database
                    </p>
                    <Button className="mt-4" variant="outline">Upsert Data</Button>
                  </div>
                  
                  <div className="rounded-md border p-4">
                    <h3 className="font-medium">Search Vectors</h3>
                    <p className="mt-1 text-sm text-muted-foreground">
                      Test similarity search with your vector database
                    </p>
                    <Button className="mt-4" variant="outline">Test Search</Button>
                  </div>
                  
                  <div className="rounded-md border p-4">
                    <h3 className="font-medium">Delete Vectors</h3>
                    <p className="mt-1 text-sm text-muted-foreground">
                      Remove vectors from your database
                    </p>
                    <Button className="mt-4" variant="outline">Delete Data</Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="settings" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Vector Database Settings</CardTitle>
                <CardDescription>
                  Configure global vector database settings
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between space-y-2">
                    <div>
                      <Label htmlFor="default-dimensions">Default Dimensions</Label>
                      <p className="text-sm text-muted-foreground">
                        Set the default vector dimensions for new connections
                      </p>
                    </div>
                    <Input
                      id="default-dimensions"
                      type="number"
                      className="w-24"
                      defaultValue="1536"
                    />
                  </div>
                  
                  <div className="flex items-center justify-between space-y-2">
                    <div>
                      <Label htmlFor="default-metric">Default Metric</Label>
                      <p className="text-sm text-muted-foreground">
                        Set the default distance metric for similarity search
                      </p>
                    </div>
                    <Select defaultValue="cosine">
                      <SelectTrigger id="default-metric" className="w-40">
                        <SelectValue placeholder="Select metric" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="cosine">Cosine</SelectItem>
                        <SelectItem value="euclidean">Euclidean</SelectItem>
                        <SelectItem value="dot">Dot Product</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div className="flex items-center justify-between space-y-2">
                    <div>
                      <div className="font-medium">Cache Results</div>
                      <p className="text-sm text-muted-foreground">
                        Cache vector search results to improve performance
                      </p>
                    </div>
                    <Switch defaultChecked />
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <Button>Save Settings</Button>
              </CardFooter>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
} 