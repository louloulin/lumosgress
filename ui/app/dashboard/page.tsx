"use client"

import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { TrafficChart } from "@/components/dashboard/traffic-chart"
import { LlmUsageChart } from "@/components/dashboard/llm-usage-chart"

export default function DashboardPage() {
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Welcome to the Proksi AI Gateway control panel.
        </p>
        
        <Tabs defaultValue="overview" className="mt-6">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="analytics">Analytics</TabsTrigger>
            <TabsTrigger value="llm-usage">LLM Usage</TabsTrigger>
          </TabsList>
          <TabsContent value="overview" className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Total Requests
                  </CardTitle>
                  <ApiIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">132,456</div>
                  <p className="text-xs text-muted-foreground">
                    +12.5% from last month
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Avg. Response Time
                  </CardTitle>
                  <TimerIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">324ms</div>
                  <p className="text-xs text-muted-foreground">
                    -18ms from last week
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Active Plugins
                  </CardTitle>
                  <PlugIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">8</div>
                  <p className="text-xs text-muted-foreground">
                    +2 new this month
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Error Rate
                  </CardTitle>
                  <AlertTriangleIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">0.12%</div>
                  <p className="text-xs text-muted-foreground">
                    -0.04% from last week
                  </p>
                </CardContent>
              </Card>
            </div>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
              <Card className="col-span-4">
                <CardHeader>
                  <CardTitle>Request Traffic</CardTitle>
                  <CardDescription>
                    Request volume over the past 30 days
                  </CardDescription>
                </CardHeader>
                <CardContent className="h-[300px]">
                  <TrafficChart />
                </CardContent>
              </Card>
              <Card className="col-span-3">
                <CardHeader>
                  <CardTitle>LLM Usage</CardTitle>
                  <CardDescription>
                    Distribution by provider
                  </CardDescription>
                </CardHeader>
                <CardContent className="h-[300px]">
                  {/* Chart placeholder */}
                  <div className="flex h-full items-center justify-center rounded-md border border-dashed">
                    <div className="text-center">
                      <p className="text-muted-foreground">Chart placeholder</p>
                      <p className="text-xs text-muted-foreground">
                        (Charts will be implemented with Chart.js or Recharts)
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>Recent Events</CardTitle>
                  <CardDescription>
                    Last 5 system events
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2">
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <CircleIcon className="h-2 w-2 fill-green-500 text-green-500" />
                        <span className="font-medium">Plugin activated</span>
                      </div>
                      <span className="text-sm text-muted-foreground">5m ago</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <CircleIcon className="h-2 w-2 fill-blue-500 text-blue-500" />
                        <span className="font-medium">Config updated</span>
                      </div>
                      <span className="text-sm text-muted-foreground">1h ago</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <CircleIcon className="h-2 w-2 fill-yellow-500 text-yellow-500" />
                        <span className="font-medium">High traffic detected</span>
                      </div>
                      <span className="text-sm text-muted-foreground">3h ago</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <CircleIcon className="h-2 w-2 fill-red-500 text-red-500" />
                        <span className="font-medium">Error rate spike</span>
                      </div>
                      <span className="text-sm text-muted-foreground">6h ago</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <CircleIcon className="h-2 w-2 fill-green-500 text-green-500" />
                        <span className="font-medium">System updated</span>
                      </div>
                      <span className="text-sm text-muted-foreground">1d ago</span>
                    </li>
                  </ul>
                </CardContent>
                <CardFooter>
                  <a href="#" className="text-sm text-blue-500 hover:underline">
                    View all events
                  </a>
                </CardFooter>
              </Card>
              <Card>
                <CardHeader>
                  <CardTitle>Active Plugins</CardTitle>
                  <CardDescription>
                    Currently enabled plugins
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2">
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <PlugIcon className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">Prompt Transformer</span>
                      </div>
                      <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800">Active</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <PlugIcon className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">LLM Router</span>
                      </div>
                      <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800">Active</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <PlugIcon className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">AI Security</span>
                      </div>
                      <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800">Active</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <PlugIcon className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">Vector DB</span>
                      </div>
                      <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800">Active</span>
                    </li>
                    <li className="flex items-center justify-between rounded-md p-2 hover:bg-muted">
                      <div className="flex items-center gap-2">
                        <PlugIcon className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">Model Aggregator</span>
                      </div>
                      <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800">Active</span>
                    </li>
                  </ul>
                </CardContent>
                <CardFooter>
                  <a href="#" className="text-sm text-blue-500 hover:underline">
                    Manage plugins
                  </a>
                </CardFooter>
              </Card>
            </div>
          </TabsContent>
          <TabsContent value="analytics" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Advanced Analytics</CardTitle>
                <CardDescription>
                  Detailed performance metrics and traffic patterns
                </CardDescription>
              </CardHeader>
              <CardContent className="h-[400px]">
                <div className="flex h-full items-center justify-center rounded-md border border-dashed">
                  <div className="text-center">
                    <p className="text-muted-foreground">Analytics dashboard will be implemented here</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="llm-usage" className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Total Tokens
                  </CardTitle>
                  <SVGIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">3,349,000</div>
                  <p className="text-xs text-muted-foreground">
                    +15.2% from last month
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Avg. Token Usage
                  </CardTitle>
                  <TimerIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">1,234</div>
                  <p className="text-xs text-muted-foreground">
                    +10% from last week
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Cost
                </CardTitle>
                  <DollarIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">$1,234.56</div>
                  <p className="text-xs text-muted-foreground">
                    +$123.45 from last month
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Error Rate
                  </CardTitle>
                  <AlertTriangleIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">0.12%</div>
                  <p className="text-xs text-muted-foreground">
                    -0.04% from last week
                  </p>
                </CardContent>
              </Card>
            </div>
            
            <Card>
              <CardHeader>
                <CardTitle>LLM Provider Usage</CardTitle>
                <CardDescription>
                  Request distribution across LLM providers
                </CardDescription>
              </CardHeader>
              <CardContent className="h-[360px]">
                <LlmUsageChart />
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
}

function ApiIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </svg>
  )
}

function TimerIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <line x1="10" x2="14" y1="2" y2="2" />
      <line x1="12" x2="15" y1="14" y2="11" />
      <circle cx="12" cy="14" r="8" />
    </svg>
  )
}

function PlugIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M12 22v-5" />
      <path d="M9 8V2" />
      <path d="M15 8V2" />
      <path d="M18 8v4a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />
    </svg>
  )
}

function AlertTriangleIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
      <path d="M12 9v4" />
      <path d="M12 17h.01" />
    </svg>
  )
}

function CircleIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <circle cx="12" cy="12" r="10" />
    </svg>
  )
}

function SVGIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M12 22v-5" />
      <path d="M9 8V2" />
      <path d="M15 8V2" />
      <path d="M18 8v4a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />
    </svg>
  )
}

function DollarIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M12 22v-5" />
      <path d="M9 8V2" />
      <path d="M15 8V2" />
      <path d="M18 8v4a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />
    </svg>
  )
} 