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

// Define the route type
interface Route {
  id: string
  name: string
  path: string
  target: string
  method: string
  active: boolean
  type: 'llm' | 'vector' | 'other'
}

export default function RoutesPage() {
  const [activeTab, setActiveTab] = useState("all")
  
  // Mock data - would come from API in real implementation
  const routes: Route[] = [
    {
      id: "1",
      name: "OpenAI Completions",
      path: "/v1/completions",
      target: "https://api.openai.com/v1/completions",
      method: "POST",
      active: true,
      type: "llm"
    },
    {
      id: "2",
      name: "OpenAI Chat",
      path: "/v1/chat/completions",
      target: "https://api.openai.com/v1/chat/completions",
      method: "POST",
      active: true,
      type: "llm"
    },
    {
      id: "3",
      name: "Anthropic Messages",
      path: "/v1/messages",
      target: "https://api.anthropic.com/v1/messages",
      method: "POST",
      active: true,
      type: "llm"
    },
    {
      id: "4",
      name: "Vector Search",
      path: "/vectors/search",
      target: "INTERNAL",
      method: "POST",
      active: true,
      type: "vector"
    },
    {
      id: "5",
      name: "Vector Upsert",
      path: "/vectors/upsert",
      target: "INTERNAL",
      method: "POST",
      active: true,
      type: "vector"
    }
  ]
  
  // Filter routes based on active tab
  const filteredRoutes = activeTab === "all" 
    ? routes 
    : routes.filter(route => route.type === activeTab)
  
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Routes Management</h1>
            <p className="text-muted-foreground">
              Configure and manage your AI Gateway routes
            </p>
          </div>
          <Button>
            <PlusIcon className="mr-2 h-4 w-4" />
            Add New Route
          </Button>
        </div>
        
        <Card className="mt-6">
          <CardHeader>
            <CardTitle>Routes</CardTitle>
            <CardDescription>
              Manage route configurations for your AI Gateway
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center space-x-2">
                <Input 
                  placeholder="Search routes..." 
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
                  </SelectContent>
                </Select>
              </div>
              
              <Tabs defaultValue="all" className="w-full" onValueChange={setActiveTab}>
                <TabsList>
                  <TabsTrigger value="all">All Routes</TabsTrigger>
                  <TabsTrigger value="llm">LLM Routes</TabsTrigger>
                  <TabsTrigger value="vector">Vector Routes</TabsTrigger>
                  <TabsTrigger value="other">Other Routes</TabsTrigger>
                </TabsList>
                
                <TabsContent value="all" className="mt-4">
                  <RoutesList routes={filteredRoutes} />
                </TabsContent>
                <TabsContent value="llm" className="mt-4">
                  <RoutesList routes={filteredRoutes} />
                </TabsContent>
                <TabsContent value="vector" className="mt-4">
                  <RoutesList routes={filteredRoutes} />
                </TabsContent>
                <TabsContent value="other" className="mt-4">
                  <RoutesList routes={filteredRoutes} />
                </TabsContent>
              </Tabs>
            </div>
          </CardContent>
        </Card>
        
        <Card className="mt-4">
          <CardHeader>
            <CardTitle>Route Configuration Editor</CardTitle>
            <CardDescription>
              Create or edit route configurations
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="route-name">Route Name</Label>
                  <Input id="route-name" placeholder="OpenAI Completions" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="route-path">Path</Label>
                  <Input id="route-path" placeholder="/v1/completions" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="route-target">Target URL</Label>
                  <Input id="route-target" placeholder="https://api.openai.com/v1/completions" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="route-method">HTTP Method</Label>
                  <Select defaultValue="POST">
                    <SelectTrigger id="route-method">
                      <SelectValue placeholder="Select method" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="GET">GET</SelectItem>
                      <SelectItem value="POST">POST</SelectItem>
                      <SelectItem value="PUT">PUT</SelectItem>
                      <SelectItem value="DELETE">DELETE</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="route-type">Route Type</Label>
                  <Select defaultValue="llm">
                    <SelectTrigger id="route-type">
                      <SelectValue placeholder="Select type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="llm">LLM</SelectItem>
                      <SelectItem value="vector">Vector</SelectItem>
                      <SelectItem value="other">Other</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-center space-x-2 pt-8">
                  <Switch id="route-active" defaultChecked />
                  <Label htmlFor="route-active">Active</Label>
                </div>
              </div>
              
              <div className="pt-4 flex justify-end space-x-2">
                <Button variant="outline">Cancel</Button>
                <Button>Save Route</Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  )
}

// Routes list component
function RoutesList({ routes }: { routes: Route[] }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>Path</TableHead>
          <TableHead>Target</TableHead>
          <TableHead>Method</TableHead>
          <TableHead>Type</TableHead>
          <TableHead>Status</TableHead>
          <TableHead className="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {routes.map(route => (
          <TableRow key={route.id}>
            <TableCell className="font-medium">{route.name}</TableCell>
            <TableCell>{route.path}</TableCell>
            <TableCell>{route.target}</TableCell>
            <TableCell>{route.method}</TableCell>
            <TableCell>
              <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium
                ${route.type === 'llm' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300' : 
                  route.type === 'vector' ? 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300' : 
                  'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300'}`}>
                {route.type}
              </span>
            </TableCell>
            <TableCell>
              <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                route.active 
                  ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300' 
                  : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
              }`}>
                {route.active ? 'Active' : 'Inactive'}
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

// Icon components
function PlusIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M5 12h14" />
      <path d="M12 5v14" />
    </svg>
  )
}

function PencilIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
      <path d="m15 5 4 4" />
    </svg>
  )
}

function TrashIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M3 6h18" />
      <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
      <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
    </svg>
  )
} 