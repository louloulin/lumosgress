// 租户和隔离设置类型定义
export interface ResourceQuota {
  requests: number;
  tokens: number;
}

export interface ResourceUsage {
  requests: number;
  tokens: number;
}

export interface Tenant {
  id: string;
  name: string;
  plan: string;
  status: string;
  users: number;
  quota: ResourceQuota;
  usage: ResourceUsage;
}

export interface IsolationSettings {
  data_isolation: boolean;
  endpoint_isolation: boolean;
  compute_isolation: boolean;
  network_isolation: boolean;
  resource_quotas: boolean;
  default_quota: ResourceQuota;
}

// 模拟数据
const mockTenants: Tenant[] = [
  { 
    id: "1", 
    name: "Enterprise Corp", 
    plan: "enterprise", 
    status: "active", 
    users: 15, 
    quota: { requests: 100000, tokens: 5000000 },
    usage: { requests: 45621, tokens: 2123456 }
  },
  { 
    id: "2", 
    name: "StartupXYZ", 
    plan: "business", 
    status: "active", 
    users: 5, 
    quota: { requests: 50000, tokens: 2000000 },
    usage: { requests: 32145, tokens: 1542321 }
  },
  { 
    id: "3", 
    name: "DevTeam Alpha", 
    plan: "developer", 
    status: "active", 
    users: 3, 
    quota: { requests: 10000, tokens: 500000 },
    usage: { requests: 8754, tokens: 321543 }
  },
  { 
    id: "4", 
    name: "Research Labs", 
    plan: "business", 
    status: "suspended", 
    users: 7, 
    quota: { requests: 50000, tokens: 2000000 },
    usage: { requests: 0, tokens: 0 }
  }
];

const mockIsolationSettings: IsolationSettings = {
  data_isolation: true,
  endpoint_isolation: true,
  compute_isolation: false,
  network_isolation: false,
  resource_quotas: true,
  default_quota: {
    requests: 10000,
    tokens: 500000
  }
};

// 内存中的租户存储
let tenants = [...mockTenants];
let isolationSettings = {...mockIsolationSettings};

// 租户 API 函数（模拟实现）
export async function getTenants(): Promise<Tenant[]> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  return [...tenants];
}

export async function getTenant(id: string): Promise<Tenant> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  
  const tenant = tenants.find(t => t.id === id);
  if (!tenant) {
    throw new Error('Tenant not found');
  }
  
  return {...tenant};
}

export async function createTenant(tenant: Omit<Tenant, 'id' | 'usage'>): Promise<Tenant> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  
  const newTenant: Tenant = {
    id: Date.now().toString(),
    ...tenant,
    usage: {
      requests: 0,
      tokens: 0
    }
  };
  
  tenants.push(newTenant);
  return {...newTenant};
}

export async function updateTenant(id: string, tenant: Tenant): Promise<Tenant> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  
  const index = tenants.findIndex(t => t.id === id);
  if (index === -1) {
    throw new Error('Tenant not found');
  }
  
  tenants[index] = {...tenant};
  return {...tenant};
}

export async function toggleTenantStatus(id: string): Promise<Tenant> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  
  const index = tenants.findIndex(t => t.id === id);
  if (index === -1) {
    throw new Error('Tenant not found');
  }
  
  const updatedTenant = {...tenants[index]};
  updatedTenant.status = updatedTenant.status === 'active' ? 'suspended' : 'active';
  
  tenants[index] = updatedTenant;
  return updatedTenant;
}

export async function getIsolationSettings(): Promise<IsolationSettings> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  
  return {...isolationSettings};
}

export async function updateIsolationSettings(settings: IsolationSettings): Promise<IsolationSettings> {
  // 模拟 API 延迟
  await new Promise(resolve => setTimeout(resolve, 500));
  
  isolationSettings = {...settings};
  return {...settings};
} 