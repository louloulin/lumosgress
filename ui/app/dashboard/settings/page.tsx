"use client"

import { useState } from "react"
import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { useAuth } from "@/lib/auth-provider"

// Mock user data
const mockUsers = [
  { id: "1", name: "Admin User", email: "admin@example.com", role: "admin", active: true },
  { id: "2", name: "Dev User", email: "dev@example.com", role: "developer", active: true },
  { id: "3", name: "API User", email: "api@example.com", role: "api", active: false },
]

export default function SettingsPage() {
  const { user } = useAuth()
  const [activeTab, setActiveTab] = useState("system")
  const [theme, setTheme] = useState("system")
  const [users, setUsers] = useState(mockUsers)
  const [metrics, setMetrics] = useState({
    collectAnonymousUsage: true,
    errorReporting: true,
    performanceMetrics: true
  })
  const [notifications, setNotifications] = useState({
    email: true,
    slack: false,
    webhook: false
  })
  
  // System settings
  const [systemSettings, setSystemSettings] = useState({
    cacheEnabled: true,
    cacheTTL: "3600",
    maxConcurrentRequests: "100",
    timeoutSeconds: "30",
    rateLimitRequests: "100",
    rateLimitInterval: "minute"
  })

  // Toggle user active status
  const toggleUserStatus = (userId: string) => {
    setUsers(users.map(user => 
      user.id === userId ? { ...user, active: !user.active } : user
    ))
  }

  return (
    <DashboardLayout>
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Settings</h1>
            <p className="text-muted-foreground">
              Configure your AI Gateway and user preferences
            </p>
          </div>
        </div>
        
        <Tabs value={activeTab} onValueChange={setActiveTab} className="mt-6">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="system">System</TabsTrigger>
            <TabsTrigger value="users">Users & Permissions</TabsTrigger>
            <TabsTrigger value="appearance">Appearance</TabsTrigger>
            <TabsTrigger value="notifications">Notifications</TabsTrigger>
          </TabsList>
          
          {/* System Settings */}
          <TabsContent value="system" className="space-y-4 mt-4">
            <Card>
              <CardHeader>
                <CardTitle>System Configuration</CardTitle>
                <CardDescription>
                  Configure core AI Gateway settings
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="cache-enabled">Cache</Label>
                    <div className="flex items-center space-x-2">
                      <Switch 
                        id="cache-enabled" 
                        checked={systemSettings.cacheEnabled}
                        onCheckedChange={(checked) => setSystemSettings({...systemSettings, cacheEnabled: checked})}
                      />
                      <Label htmlFor="cache-enabled">Enable response caching</Label>
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="cache-ttl">Cache TTL (seconds)</Label>
                    <Input 
                      id="cache-ttl" 
                      type="number" 
                      value={systemSettings.cacheTTL}
                      onChange={(e) => setSystemSettings({...systemSettings, cacheTTL: e.target.value})}
                      disabled={!systemSettings.cacheEnabled}
                    />
                  </div>
                </div>
                
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="max-requests">Max Concurrent Requests</Label>
                    <Input 
                      id="max-requests" 
                      type="number" 
                      value={systemSettings.maxConcurrentRequests}
                      onChange={(e) => setSystemSettings({...systemSettings, maxConcurrentRequests: e.target.value})}
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="timeout">Request Timeout (seconds)</Label>
                    <Input 
                      id="timeout" 
                      type="number" 
                      value={systemSettings.timeoutSeconds}
                      onChange={(e) => setSystemSettings({...systemSettings, timeoutSeconds: e.target.value})}
                    />
                  </div>
                </div>
                
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="rate-limit">Rate Limit (requests)</Label>
                    <Input 
                      id="rate-limit" 
                      type="number" 
                      value={systemSettings.rateLimitRequests}
                      onChange={(e) => setSystemSettings({...systemSettings, rateLimitRequests: e.target.value})}
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="rate-interval">Rate Limit Interval</Label>
                    <Select 
                      value={systemSettings.rateLimitInterval}
                      onValueChange={(value) => setSystemSettings({...systemSettings, rateLimitInterval: value})}
                    >
                      <SelectTrigger id="rate-interval">
                        <SelectValue placeholder="Select interval" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="second">Per Second</SelectItem>
                        <SelectItem value="minute">Per Minute</SelectItem>
                        <SelectItem value="hour">Per Hour</SelectItem>
                        <SelectItem value="day">Per Day</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                
                <div className="space-y-2">
                  <Label>Telemetry</Label>
                  <div className="space-y-2">
                    <div className="flex items-center space-x-2">
                      <Switch 
                        id="collect-usage" 
                        checked={metrics.collectAnonymousUsage}
                        onCheckedChange={(checked) => setMetrics({...metrics, collectAnonymousUsage: checked})}
                      />
                      <Label htmlFor="collect-usage">Collect anonymous usage data</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Switch 
                        id="error-reporting" 
                        checked={metrics.errorReporting}
                        onCheckedChange={(checked) => setMetrics({...metrics, errorReporting: checked})}
                      />
                      <Label htmlFor="error-reporting">Enable error reporting</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Switch 
                        id="performance-metrics" 
                        checked={metrics.performanceMetrics}
                        onCheckedChange={(checked) => setMetrics({...metrics, performanceMetrics: checked})}
                      />
                      <Label htmlFor="performance-metrics">Collect performance metrics</Label>
                    </div>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <Button>Save System Settings</Button>
              </CardFooter>
            </Card>
          </TabsContent>
          
          {/* Users & Permissions */}
          <TabsContent value="users" className="space-y-4 mt-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between">
                <div>
                  <CardTitle>User Management</CardTitle>
                  <CardDescription>
                    Manage users and their permissions
                  </CardDescription>
                </div>
                <Button>Add User</Button>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Name</TableHead>
                      <TableHead>Email</TableHead>
                      <TableHead>Role</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead>Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {users.map((user) => (
                      <TableRow key={user.id}>
                        <TableCell>{user.name}</TableCell>
                        <TableCell>{user.email}</TableCell>
                        <TableCell className="capitalize">{user.role}</TableCell>
                        <TableCell>
                          <div className="flex items-center space-x-2">
                            <Switch 
                              checked={user.active} 
                              onCheckedChange={() => toggleUserStatus(user.id)}
                            />
                            <span>{user.active ? "Active" : "Inactive"}</span>
                          </div>
                        </TableCell>
                        <TableCell>
                          <Button variant="ghost" size="sm">Edit</Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
            
            <Card>
              <CardHeader>
                <CardTitle>Role Permissions</CardTitle>
                <CardDescription>
                  Configure permissions for each user role
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Admin</h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      Full system access and user management
                    </p>
                    <div className="grid grid-cols-2 gap-2">
                      <div className="flex items-center space-x-2">
                        <Switch id="admin-system" checked disabled />
                        <Label htmlFor="admin-system">System settings</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="admin-users" checked disabled />
                        <Label htmlFor="admin-users">User management</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="admin-plugins" checked disabled />
                        <Label htmlFor="admin-plugins">Plugin management</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="admin-routes" checked disabled />
                        <Label htmlFor="admin-routes">Route management</Label>
                      </div>
                    </div>
                  </div>
                  
                  <div>
                    <h3 className="font-semibold mb-2">Developer</h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      Can manage routes and test API functionality
                    </p>
                    <div className="grid grid-cols-2 gap-2">
                      <div className="flex items-center space-x-2">
                        <Switch id="dev-system" />
                        <Label htmlFor="dev-system">System settings</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="dev-users" />
                        <Label htmlFor="dev-users">User management</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="dev-plugins" checked />
                        <Label htmlFor="dev-plugins">Plugin management</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="dev-routes" checked />
                        <Label htmlFor="dev-routes">Route management</Label>
                      </div>
                    </div>
                  </div>
                  
                  <div>
                    <h3 className="font-semibold mb-2">API</h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      API-only access for integrations
                    </p>
                    <div className="grid grid-cols-2 gap-2">
                      <div className="flex items-center space-x-2">
                        <Switch id="api-system" />
                        <Label htmlFor="api-system">System settings</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="api-users" />
                        <Label htmlFor="api-users">User management</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="api-plugins" />
                        <Label htmlFor="api-plugins">Plugin management</Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="api-routes" />
                        <Label htmlFor="api-routes">Route management</Label>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <Button>Save Permissions</Button>
              </CardFooter>
            </Card>
          </TabsContent>
          
          {/* Appearance */}
          <TabsContent value="appearance" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle>Appearance Settings</CardTitle>
                <CardDescription>
                  Customize the look and feel of your dashboard
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="theme">Theme</Label>
                  <Select 
                    value={theme} 
                    onValueChange={setTheme}
                  >
                    <SelectTrigger id="theme">
                      <SelectValue placeholder="Select theme" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="light">Light</SelectItem>
                      <SelectItem value="dark">Dark</SelectItem>
                      <SelectItem value="system">System</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="density">Density</Label>
                  <Select defaultValue="comfortable">
                    <SelectTrigger id="density">
                      <SelectValue placeholder="Select density" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="compact">Compact</SelectItem>
                      <SelectItem value="comfortable">Comfortable</SelectItem>
                      <SelectItem value="spacious">Spacious</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="animation">Animations</Label>
                  <Select defaultValue="enabled">
                    <SelectTrigger id="animation">
                      <SelectValue placeholder="Select animation setting" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="enabled">Enabled</SelectItem>
                      <SelectItem value="reduced">Reduced</SelectItem>
                      <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="font-size">Font Size</Label>
                  <Select defaultValue="medium">
                    <SelectTrigger id="font-size">
                      <SelectValue placeholder="Select font size" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="small">Small</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="large">Large</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
              <CardFooter>
                <Button>Save Appearance</Button>
              </CardFooter>
            </Card>
          </TabsContent>
          
          {/* Notifications */}
          <TabsContent value="notifications" className="space-y-4 mt-4">
            <Card>
              <CardHeader>
                <CardTitle>Notification Settings</CardTitle>
                <CardDescription>
                  Configure how you receive alerts and notifications
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <h3 className="font-semibold">Notification Channels</h3>
                  <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label htmlFor="email-notifications">Email Notifications</Label>
                        <p className="text-sm text-muted-foreground">
                          Receive alerts via email
                        </p>
                      </div>
                      <Switch 
                        id="email-notifications" 
                        checked={notifications.email}
                        onCheckedChange={(checked) => setNotifications({...notifications, email: checked})}
                      />
                    </div>
                    
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label htmlFor="slack-notifications">Slack Integration</Label>
                        <p className="text-sm text-muted-foreground">
                          Send alerts to Slack channel
                        </p>
                      </div>
                      <Switch 
                        id="slack-notifications" 
                        checked={notifications.slack}
                        onCheckedChange={(checked) => setNotifications({...notifications, slack: checked})}
                      />
                    </div>
                    
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label htmlFor="webhook-notifications">Webhook</Label>
                        <p className="text-sm text-muted-foreground">
                          Send alerts to custom webhook endpoint
                        </p>
                      </div>
                      <Switch 
                        id="webhook-notifications" 
                        checked={notifications.webhook}
                        onCheckedChange={(checked) => setNotifications({...notifications, webhook: checked})}
                      />
                    </div>
                  </div>
                </div>
                
                {notifications.email && (
                  <div className="space-y-2">
                    <Label htmlFor="email-address">Email Address</Label>
                    <Input id="email-address" type="email" placeholder="admin@example.com" />
                  </div>
                )}
                
                {notifications.slack && (
                  <div className="space-y-2">
                    <Label htmlFor="slack-webhook">Slack Webhook URL</Label>
                    <Input id="slack-webhook" type="url" placeholder="https://hooks.slack.com/services/..." />
                    <Label htmlFor="slack-channel">Channel</Label>
                    <Input id="slack-channel" placeholder="#alerts" />
                  </div>
                )}
                
                {notifications.webhook && (
                  <div className="space-y-2">
                    <Label htmlFor="webhook-url">Webhook URL</Label>
                    <Input id="webhook-url" type="url" placeholder="https://api.example.com/webhooks/alerts" />
                  </div>
                )}
                
                <div className="space-y-2">
                  <h3 className="font-semibold">Alert Types</h3>
                  <div className="space-y-2">
                    <div className="flex items-center space-x-2">
                      <Switch id="system-alerts" defaultChecked />
                      <Label htmlFor="system-alerts">System Alerts</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Switch id="security-alerts" defaultChecked />
                      <Label htmlFor="security-alerts">Security Alerts</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Switch id="performance-alerts" defaultChecked />
                      <Label htmlFor="performance-alerts">Performance Alerts</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Switch id="usage-alerts" defaultChecked />
                      <Label htmlFor="usage-alerts">Usage Alerts</Label>
                    </div>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <Button>Save Notification Settings</Button>
              </CardFooter>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
} 