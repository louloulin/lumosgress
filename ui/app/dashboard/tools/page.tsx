"use client"

import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import Link from "next/link"
import { Button } from "@/components/ui/button"

export default function DeveloperToolsPage() {
  const tools = [
    {
      title: "API Configuration",
      description: "Manage API endpoints, keys, and test API requests",
      href: "/dashboard/tools/api-config",
      icon: ApiIcon,
    },
    {
      title: "Request Builder",
      description: "Build and test AI API requests to different providers",
      href: "/dashboard/tools/request-builder", 
      icon: RequestIcon,
    },
    {
      title: "Prompt Debugger",
      description: "Debug and optimize your prompts for better results",
      href: "/dashboard/tools/prompt-debugger",
      icon: DebugIcon,
      comingSoon: false,
    },
    {
      title: "SDK Documentation",
      description: "Explore SDK documentation and client libraries",
      href: "/dashboard/tools/sdk-docs",
      icon: DocIcon,
      comingSoon: true,
    },
  ]
  
  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div>
          <h1 className="text-3xl font-bold">Developer Tools</h1>
          <p className="text-muted-foreground">
            Tools to help you build, test, and optimize your AI integrations
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-6">
          {tools.map((tool) => (
            <Card key={tool.title} className={tool.comingSoon ? "opacity-70" : ""}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <tool.icon className="h-8 w-8 text-primary" />
                  {tool.comingSoon && (
                    <span className="text-xs bg-primary/20 text-primary px-2 py-1 rounded-full">
                      Coming Soon
                    </span>
                  )}
                </div>
                <CardTitle className="mt-4">{tool.title}</CardTitle>
                <CardDescription>{tool.description}</CardDescription>
              </CardHeader>
              <CardFooter>
                {tool.comingSoon ? (
                  <Button variant="outline" className="w-full" disabled>
                    Coming Soon
                  </Button>
                ) : (
                  <Link href={tool.href} className="w-full">
                    <Button variant="default" className="w-full">
                      Open Tool
                    </Button>
                  </Link>
                )}
              </CardFooter>
            </Card>
          ))}
        </div>
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
      <path d="M12 3h7a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-7m0-18H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h7m0-18v18" />
      <path d="M8 7h4" />
      <path d="M8 17h4" />
      <path d="M16 7h.01" />
      <path d="M16 17h.01" />
    </svg>
  )
}

function RequestIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="m22 2-7 20-4-9-9-4Z" />
      <path d="M22 2 11 13" />
    </svg>
  )
}

function DebugIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M17 8h1a4 4 0 1 1 0 8h-1" />
      <path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z" />
      <line x1="6" x2="6" y1="2" y2="4" />
      <line x1="10" x2="10" y1="2" y2="4" />
      <line x1="14" x2="14" y1="2" y2="4" />
    </svg>
  )
}

function DocIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" />
      <path d="M8 7h6" />
      <path d="M8 11h8" />
      <path d="M8 15h6" />
    </svg>
  )
} 