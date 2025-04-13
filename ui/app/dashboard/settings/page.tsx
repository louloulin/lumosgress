"use client"

import { useState, useEffect } from "react"
import { DashboardLayout } from "@/components/layout/dashboard-layout"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { useToast } from "@/components/ui/use-toast"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import {
  getTenants,
  getIsolationSettings,
  toggleTenantStatus,
  updateIsolationSettings,
  createTenant,
  updateTenant,
  type Tenant,
  type IsolationSettings
} from "@/lib/api/tenant"
import { Slider } from "@/components/ui/slider"
import { Badge } from "@/components/ui/badge"

// Mock user data
const mockUsers = [
  { id: "1", name: "Admin User", email: "admin@example.com", role: "admin", active: true },
  { id: "2", name: "Dev User", email: "dev@example.com", role: "developer", active: true },
  { id: "3", name: "API User", email: "api@example.com", role: "api", active: false },
]

export default function SettingsPage() {
  // const { user } = useAuth() // 暂时注释未使用的变量
  const { toast } = useToast()
  const [activeTab, setActiveTab] = useState("system")
  const [theme, setTheme] = useState("system")
  const [users, setUsers] = useState(mockUsers)
  
  // 添加缺失的状态
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
  
  // 租户状态
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [isLoadingTenants, setIsLoadingTenants] = useState(true)
  const [isolationSettings, setIsolationSettings] = useState<IsolationSettings | null>(null)
  const [isLoadingSettings, setIsLoadingSettings] = useState(true)
  
  // 添加租户对话框状态
  const [showAddTenantDialog, setShowAddTenantDialog] = useState(false)
  const [newTenant, setNewTenant] = useState({
    name: "",
    plan: "developer",
    users: 1,
    quota: { requests: 10000, tokens: 500000 }
  })
  
  // 编辑租户对话框状态
  const [showEditTenantDialog, setShowEditTenantDialog] = useState(false)
  const [editingTenant, setEditingTenant] = useState<Tenant | null>(null)

  // 加载租户数据
  useEffect(() => {
    const loadTenants = async () => {
      try {
        setIsLoadingTenants(true)
        const data = await getTenants()
        setTenants(data)
      } catch (error) {
        console.error("Failed to load tenants:", error)
        toast({
          title: "Error",
          description: "Failed to load tenants. Please try again.",
          variant: "destructive"
        })
      } finally {
        setIsLoadingTenants(false)
      }
    }
    
    loadTenants()
  }, [toast])
  
  // 加载隔离设置
  useEffect(() => {
    const loadIsolationSettings = async () => {
      try {
        setIsLoadingSettings(true)
        const data = await getIsolationSettings()
        setIsolationSettings(data)
      } catch (error) {
        console.error("Failed to load isolation settings:", error)
        toast({
          title: "Error",
          description: "Failed to load isolation settings. Please try again.",
          variant: "destructive"
        })
      } finally {
        setIsLoadingSettings(false)
      }
    }
    
    loadIsolationSettings()
  }, [toast])

  // Toggle user active status
  const toggleUserStatus = (userId: string) => {
    setUsers(users.map(user => 
      user.id === userId ? { ...user, active: !user.active } : user
    ))
  }

  // Toggle tenant status
  const handleToggleTenantStatus = async (tenantId: string) => {
    try {
      const updatedTenant = await toggleTenantStatus(tenantId)
      setTenants(tenants.map(tenant => 
        tenant.id === tenantId ? updatedTenant : tenant
      ))
      
      toast({
        title: "Success",
        description: `Tenant status updated to ${updatedTenant.status}.`,
      })
    } catch (error) {
      console.error("Failed to toggle tenant status:", error)
      toast({
        title: "Error",
        description: "Failed to update tenant status. Please try again.",
        variant: "destructive"
      })
    }
  }
  
  // Save isolation settings
  const handleSaveIsolationSettings = async () => {
    if (!isolationSettings) return
    
    try {
      const updatedSettings = await updateIsolationSettings(isolationSettings)
      setIsolationSettings(updatedSettings)
      
      toast({
        title: "Success",
        description: "Isolation settings saved successfully.",
      })
    } catch (error) {
      console.error("Failed to save isolation settings:", error)
      toast({
        title: "Error",
        description: "Failed to save isolation settings. Please try again.",
        variant: "destructive"
      })
    }
  }
  
  // Handle adding new tenant
  const handleAddTenant = async () => {
    try {
      const tenant = await createTenant({
        name: newTenant.name,
        plan: newTenant.plan,
        status: "active",
        users: newTenant.users,
        quota: newTenant.quota
      })
      
      setTenants([...tenants, tenant])
      setShowAddTenantDialog(false)
      setNewTenant({
        name: "",
        plan: "developer",
        users: 1,
        quota: { requests: 10000, tokens: 500000 }
      })
      
      toast({
        title: "Success",
        description: "Tenant added successfully.",
      })
    } catch (error) {
      console.error("Failed to add tenant:", error)
      toast({
        title: "Error",
        description: "Failed to add tenant. Please try again.",
        variant: "destructive"
      })
    }
  }
  
  // Handle editing tenant
  const handleEditTenant = async () => {
    if (!editingTenant) return
    
    try {
      const updatedTenant = await updateTenant(editingTenant.id, editingTenant)
      setTenants(tenants.map(tenant => 
        tenant.id === updatedTenant.id ? updatedTenant : tenant
      ))
      setShowEditTenantDialog(false)
      setEditingTenant(null)
      
      toast({
        title: "Success",
        description: "Tenant updated successfully.",
      })
    } catch (error) {
      console.error("Failed to update tenant:", error)
      toast({
        title: "Error",
        description: "Failed to update tenant. Please try again.",
        variant: "destructive"
      })
    }
  }
  
  // Start editing tenant
  const startEditTenant = (tenant: Tenant) => {
    setEditingTenant({ ...tenant })
    setShowEditTenantDialog(true)
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
          <TabsList className="grid w-full grid-cols-5">
            <TabsTrigger value="system">System</TabsTrigger>
            <TabsTrigger value="users">Users & Permissions</TabsTrigger>
            <TabsTrigger value="tenants">Multi-tenant</TabsTrigger>
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
          
          {/* Multi-tenant Isolation */}
          <TabsContent value="tenants" className="space-y-4 mt-4">
            <Card>
              <CardHeader>
                <CardTitle>Multi-tenant Isolation</CardTitle>
                <CardDescription>
                  Configure isolation settings for multi-tenant deployments
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {isLoadingSettings ? (
                  <div className="flex justify-center py-4">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                  </div>
                ) : isolationSettings ? (
                  <>
                    <div className="space-y-4">
                      <h3 className="text-lg font-medium">Isolation Settings</h3>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <div>
                              <Label htmlFor="data-isolation">Data Isolation</Label>
                              <p className="text-sm text-muted-foreground">
                                Separate data storage for each tenant
                              </p>
                            </div>
                            <Switch 
                              id="data-isolation" 
                              checked={isolationSettings.data_isolation}
                              onCheckedChange={(checked) => 
                                setIsolationSettings({...isolationSettings, data_isolation: checked})
                              }
                            />
                          </div>
                        </div>
                        
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <div>
                              <Label htmlFor="endpoint-isolation">Endpoint Isolation</Label>
                              <p className="text-sm text-muted-foreground">
                                Dedicated API endpoints per tenant
                              </p>
                            </div>
                            <Switch 
                              id="endpoint-isolation" 
                              checked={isolationSettings.endpoint_isolation}
                              onCheckedChange={(checked) => 
                                setIsolationSettings({...isolationSettings, endpoint_isolation: checked})
                              }
                            />
                          </div>
                        </div>
                        
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <div>
                              <Label htmlFor="compute-isolation">Compute Isolation</Label>
                              <p className="text-sm text-muted-foreground">
                                Dedicated compute resources per tenant
                              </p>
                            </div>
                            <Switch 
                              id="compute-isolation" 
                              checked={isolationSettings.compute_isolation}
                              onCheckedChange={(checked) => 
                                setIsolationSettings({...isolationSettings, compute_isolation: checked})
                              }
                            />
                          </div>
                        </div>
                        
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <div>
                              <Label htmlFor="network-isolation">Network Isolation</Label>
                              <p className="text-sm text-muted-foreground">
                                Separate network configuration per tenant
                              </p>
                            </div>
                            <Switch 
                              id="network-isolation" 
                              checked={isolationSettings.network_isolation}
                              onCheckedChange={(checked) => 
                                setIsolationSettings({...isolationSettings, network_isolation: checked})
                              }
                            />
                          </div>
                        </div>
                      </div>
                    </div>

                    <div className="space-y-4">
                      <h3 className="text-lg font-medium">Resource Quotas</h3>
                      <div className="space-y-4">
                        <div className="flex items-center space-x-2">
                          <Switch 
                            id="resource-quotas" 
                            checked={isolationSettings.resource_quotas}
                            onCheckedChange={(checked) => 
                              setIsolationSettings({...isolationSettings, resource_quotas: checked})
                            }
                          />
                          <Label htmlFor="resource-quotas">Enable resource quotas for tenants</Label>
                        </div>
                        
                        {isolationSettings.resource_quotas && (
                          <div className="space-y-4">
                            <div className="space-y-2">
                              <div className="flex items-center justify-between">
                                <Label htmlFor="default-request-quota">Default Request Quota</Label>
                                <span className="text-sm font-medium">{isolationSettings.default_quota.requests.toLocaleString()} requests/month</span>
                              </div>
                              <Slider 
                                id="default-request-quota"
                                defaultValue={[isolationSettings.default_quota.requests]}
                                max={1000000}
                                step={1000}
                                onValueChange={(value) => 
                                  setIsolationSettings({
                                    ...isolationSettings, 
                                    default_quota: {...isolationSettings.default_quota, requests: value[0]}
                                  })
                                }
                              />
                            </div>
                            
                            <div className="space-y-2">
                              <div className="flex items-center justify-between">
                                <Label htmlFor="default-token-quota">Default Token Quota</Label>
                                <span className="text-sm font-medium">{isolationSettings.default_quota.tokens.toLocaleString()} tokens/month</span>
                              </div>
                              <Slider 
                                id="default-token-quota"
                                defaultValue={[isolationSettings.default_quota.tokens]}
                                max={10000000}
                                step={10000}
                                onValueChange={(value) => 
                                  setIsolationSettings({
                                    ...isolationSettings, 
                                    default_quota: {...isolationSettings.default_quota, tokens: value[0]}
                                  })
                                }
                              />
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  </>
                ) : (
                  <p>No isolation settings found.</p>
                )}
              </CardContent>
              <CardFooter>
                <Button onClick={handleSaveIsolationSettings} disabled={isLoadingSettings || !isolationSettings}>
                  {isLoadingSettings ? "Loading..." : "Save Isolation Settings"}
                </Button>
              </CardFooter>
            </Card>
            
            <Card>
              <CardHeader className="flex flex-row items-center justify-between">
                <div>
                  <CardTitle>Tenant Management</CardTitle>
                  <CardDescription>
                    Manage all tenants and their resource allocations
                  </CardDescription>
                </div>
                <Button onClick={() => setShowAddTenantDialog(true)}>Add Tenant</Button>
              </CardHeader>
              <CardContent>
                {isLoadingTenants ? (
                  <div className="flex justify-center py-4">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                  </div>
                ) : tenants.length > 0 ? (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Name</TableHead>
                        <TableHead>Plan</TableHead>
                        <TableHead>Users</TableHead>
                        <TableHead>Usage</TableHead>
                        <TableHead>Status</TableHead>
                        <TableHead>Actions</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {tenants.map((tenant) => (
                        <TableRow key={tenant.id}>
                          <TableCell className="font-medium">{tenant.name}</TableCell>
                          <TableCell className="capitalize">{tenant.plan}</TableCell>
                          <TableCell>{tenant.users}</TableCell>
                          <TableCell>
                            <div className="space-y-1">
                              <div className="flex items-center space-x-2 text-sm">
                                <span>Requests:</span>
                                <div className="w-full bg-gray-200 rounded-full h-2.5">
                                  <div 
                                    className="bg-blue-600 h-2.5 rounded-full" 
                                    style={{ width: `${Math.min(100, (tenant.usage.requests / tenant.quota.requests) * 100)}%` }}
                                  ></div>
                                </div>
                                <span className="whitespace-nowrap">{Math.round((tenant.usage.requests / tenant.quota.requests) * 100)}%</span>
                              </div>
                              <div className="flex items-center space-x-2 text-sm">
                                <span>Tokens:</span>
                                <div className="w-full bg-gray-200 rounded-full h-2.5">
                                  <div 
                                    className="bg-green-600 h-2.5 rounded-full" 
                                    style={{ width: `${Math.min(100, (tenant.usage.tokens / tenant.quota.tokens) * 100)}%` }}
                                  ></div>
                                </div>
                                <span className="whitespace-nowrap">{Math.round((tenant.usage.tokens / tenant.quota.tokens) * 100)}%</span>
                              </div>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Badge
                              className={tenant.status === "active" ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"}
                            >
                              {tenant.status === "active" ? "Active" : "Suspended"}
                            </Badge>
                          </TableCell>
                          <TableCell>
                            <div className="flex items-center space-x-2">
                              <Button variant="outline" size="sm" onClick={() => startEditTenant(tenant)}>
                                Edit
                              </Button>
                              <Button 
                                variant={tenant.status === "active" ? "destructive" : "outline"}
                                size="sm"
                                onClick={() => handleToggleTenantStatus(tenant.id)}
                              >
                                {tenant.status === "active" ? "Suspend" : "Activate"}
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                ) : (
                  <p className="text-center py-4">No tenants found. Add a tenant to get started.</p>
                )}
              </CardContent>
            </Card>
            
            {/* Add Tenant Dialog */}
            <Dialog open={showAddTenantDialog} onOpenChange={setShowAddTenantDialog}>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Add New Tenant</DialogTitle>
                  <DialogDescription>
                    Create a new tenant with resource allocation.
                  </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                  <div className="space-y-2">
                    <Label htmlFor="tenant-name">Tenant Name</Label>
                    <Input 
                      id="tenant-name" 
                      value={newTenant.name}
                      onChange={(e) => setNewTenant({...newTenant, name: e.target.value})}
                      placeholder="Enterprise Corp"
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="tenant-plan">Plan</Label>
                    <Select 
                      value={newTenant.plan}
                      onValueChange={(value) => setNewTenant({...newTenant, plan: value})}
                    >
                      <SelectTrigger id="tenant-plan">
                        <SelectValue placeholder="Select plan" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="developer">Developer</SelectItem>
                        <SelectItem value="business">Business</SelectItem>
                        <SelectItem value="enterprise">Enterprise</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="tenant-users">Number of Users</Label>
                    <Input 
                      id="tenant-users" 
                      type="number"
                      value={newTenant.users}
                      onChange={(e) => setNewTenant({...newTenant, users: parseInt(e.target.value) || 1})}
                      min="1"
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="tenant-request-quota">Request Quota</Label>
                    <div className="flex items-center space-x-2">
                      <Input 
                        id="tenant-request-quota" 
                        type="number"
                        value={newTenant.quota.requests}
                        onChange={(e) => setNewTenant({
                          ...newTenant, 
                          quota: {...newTenant.quota, requests: parseInt(e.target.value) || 0}
                        })}
                        min="0"
                      />
                      <span>requests/month</span>
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="tenant-token-quota">Token Quota</Label>
                    <div className="flex items-center space-x-2">
                      <Input 
                        id="tenant-token-quota" 
                        type="number"
                        value={newTenant.quota.tokens}
                        onChange={(e) => setNewTenant({
                          ...newTenant, 
                          quota: {...newTenant.quota, tokens: parseInt(e.target.value) || 0}
                        })}
                        min="0"
                      />
                      <span>tokens/month</span>
                    </div>
                  </div>
                </div>
                <DialogFooter>
                  <Button variant="outline" onClick={() => setShowAddTenantDialog(false)}>
                    Cancel
                  </Button>
                  <Button onClick={handleAddTenant}>
                    Add Tenant
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
            
            {/* Edit Tenant Dialog */}
            <Dialog open={showEditTenantDialog} onOpenChange={setShowEditTenantDialog}>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Edit Tenant</DialogTitle>
                  <DialogDescription>
                    Update tenant details and resource allocation.
                  </DialogDescription>
                </DialogHeader>
                {editingTenant && (
                  <div className="space-y-4 py-4">
                    <div className="space-y-2">
                      <Label htmlFor="edit-tenant-name">Tenant Name</Label>
                      <Input 
                        id="edit-tenant-name" 
                        value={editingTenant.name}
                        onChange={(e) => setEditingTenant({...editingTenant, name: e.target.value})}
                      />
                    </div>
                    
                    <div className="space-y-2">
                      <Label htmlFor="edit-tenant-plan">Plan</Label>
                      <Select 
                        value={editingTenant.plan}
                        onValueChange={(value) => setEditingTenant({...editingTenant, plan: value})}
                      >
                        <SelectTrigger id="edit-tenant-plan">
                          <SelectValue placeholder="Select plan" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="developer">Developer</SelectItem>
                          <SelectItem value="business">Business</SelectItem>
                          <SelectItem value="enterprise">Enterprise</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    
                    <div className="space-y-2">
                      <Label htmlFor="edit-tenant-users">Number of Users</Label>
                      <Input 
                        id="edit-tenant-users" 
                        type="number"
                        value={editingTenant.users}
                        onChange={(e) => setEditingTenant({...editingTenant, users: parseInt(e.target.value) || 1})}
                        min="1"
                      />
                    </div>
                    
                    <div className="space-y-2">
                      <Label htmlFor="edit-tenant-request-quota">Request Quota</Label>
                      <div className="flex items-center space-x-2">
                        <Input 
                          id="edit-tenant-request-quota" 
                          type="number"
                          value={editingTenant.quota.requests}
                          onChange={(e) => setEditingTenant({
                            ...editingTenant, 
                            quota: {...editingTenant.quota, requests: parseInt(e.target.value) || 0}
                          })}
                          min="0"
                        />
                        <span>requests/month</span>
                      </div>
                    </div>
                    
                    <div className="space-y-2">
                      <Label htmlFor="edit-tenant-token-quota">Token Quota</Label>
                      <div className="flex items-center space-x-2">
                        <Input 
                          id="edit-tenant-token-quota" 
                          type="number"
                          value={editingTenant.quota.tokens}
                          onChange={(e) => setEditingTenant({
                            ...editingTenant, 
                            quota: {...editingTenant.quota, tokens: parseInt(e.target.value) || 0}
                          })}
                          min="0"
                        />
                        <span>tokens/month</span>
                      </div>
                    </div>
                  </div>
                )}
                <DialogFooter>
                  <Button variant="outline" onClick={() => setShowEditTenantDialog(false)}>
                    Cancel
                  </Button>
                  <Button onClick={handleEditTenant}>
                    Save Changes
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
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