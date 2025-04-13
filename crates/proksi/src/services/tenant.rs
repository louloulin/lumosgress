use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

// 租户模型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub plan: String, // "developer", "business", "enterprise"
    pub status: TenantStatus,
    pub users: usize,
    pub quota: ResourceQuota,
    pub usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantStatus {
    Active,
    Suspended,
}

impl ToString for TenantStatus {
    fn to_string(&self) -> String {
        match self {
            TenantStatus::Active => "active".to_string(),
            TenantStatus::Suspended => "suspended".to_string(),
        }
    }
}

impl From<&str> for TenantStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => TenantStatus::Active,
            "suspended" => TenantStatus::Suspended,
            _ => TenantStatus::Active, // 默认为活跃状态
        }
    }
}

// 资源配额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub requests: u64,  // 每月允许的请求数
    pub tokens: u64,    // 每月允许的令牌数
}

// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub requests: u64,  // 当前已使用的请求数
    pub tokens: u64,    // 当前已使用的令牌数
}

// 隔离设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationSettings {
    pub data_isolation: bool,         // 数据隔离
    pub endpoint_isolation: bool,     // 端点隔离
    pub compute_isolation: bool,      // 计算隔离
    pub network_isolation: bool,      // 网络隔离
    pub resource_quotas: bool,        // 资源配额
    pub default_quota: ResourceQuota, // 默认配额
}

// 租户服务
#[derive(Clone)]
pub struct TenantService {
    tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    isolation_settings: Arc<RwLock<IsolationSettings>>,
}

impl TenantService {
    pub fn new() -> Self {
        let isolation_settings = IsolationSettings {
            data_isolation: true,
            endpoint_isolation: true,
            compute_isolation: false,
            network_isolation: false,
            resource_quotas: true,
            default_quota: ResourceQuota {
                requests: 10000,
                tokens: 500000,
            },
        };

        // 创建一些默认租户用于测试
        let mut tenants = HashMap::new();
        
        // 添加示例租户
        let tenant1 = Tenant {
            id: "1".to_string(),
            name: "Enterprise Corp".to_string(),
            plan: "enterprise".to_string(),
            status: TenantStatus::Active,
            users: 15,
            quota: ResourceQuota {
                requests: 100000,
                tokens: 5000000,
            },
            usage: ResourceUsage {
                requests: 45621,
                tokens: 2123456,
            },
        };
        
        tenants.insert(tenant1.id.clone(), tenant1);

        Self {
            tenants: Arc::new(RwLock::new(tenants)),
            isolation_settings: Arc::new(RwLock::new(isolation_settings)),
        }
    }

    // 获取所有租户
    pub fn get_all_tenants(&self) -> Vec<Tenant> {
        let tenants = self.tenants.read().unwrap();
        tenants.values().cloned().collect()
    }

    // 获取特定租户
    pub fn get_tenant(&self, id: &str) -> Option<Tenant> {
        let tenants = self.tenants.read().unwrap();
        tenants.get(id).cloned()
    }

    // 创建新租户
    pub fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, String> {
        let mut tenants = self.tenants.write().unwrap();
        
        // 检查租户ID是否已存在
        if tenants.contains_key(&tenant.id) {
            return Err("Tenant ID already exists".to_string());
        }
        
        tenants.insert(tenant.id.clone(), tenant.clone());
        Ok(tenant)
    }

    // 更新租户
    pub fn update_tenant(&self, id: &str, tenant: Tenant) -> Result<Tenant, String> {
        let mut tenants = self.tenants.write().unwrap();
        
        if !tenants.contains_key(id) {
            return Err("Tenant not found".to_string());
        }
        
        tenants.insert(id.to_string(), tenant.clone());
        Ok(tenant)
    }

    // 删除租户
    pub fn delete_tenant(&self, id: &str) -> Result<(), String> {
        let mut tenants = self.tenants.write().unwrap();
        
        if !tenants.contains_key(id) {
            return Err("Tenant not found".to_string());
        }
        
        tenants.remove(id);
        Ok(())
    }

    // 切换租户状态（激活/暂停）
    pub fn toggle_tenant_status(&self, id: &str) -> Result<Tenant, String> {
        let mut tenants = self.tenants.write().unwrap();
        
        if let Some(tenant) = tenants.get_mut(id) {
            tenant.status = match tenant.status {
                TenantStatus::Active => TenantStatus::Suspended,
                TenantStatus::Suspended => TenantStatus::Active,
            };
            Ok(tenant.clone())
        } else {
            Err("Tenant not found".to_string())
        }
    }

    // 获取隔离设置
    pub fn get_isolation_settings(&self) -> IsolationSettings {
        self.isolation_settings.read().unwrap().clone()
    }

    // 更新隔离设置
    pub fn update_isolation_settings(&self, settings: IsolationSettings) -> IsolationSettings {
        let mut isolation_settings = self.isolation_settings.write().unwrap();
        *isolation_settings = settings.clone();
        settings
    }

    // 记录资源使用
    pub fn record_usage(&self, tenant_id: &str, requests: u64, tokens: u64) -> Result<(), String> {
        let mut tenants = self.tenants.write().unwrap();
        
        if let Some(tenant) = tenants.get_mut(tenant_id) {
            tenant.usage.requests += requests;
            tenant.usage.tokens += tokens;
            Ok(())
        } else {
            Err("Tenant not found".to_string())
        }
    }

    // 检查是否允许使用资源
    pub fn check_quota(&self, tenant_id: &str, requests: u64, tokens: u64) -> Result<bool, String> {
        let tenants = self.tenants.read().unwrap();
        
        if let Some(tenant) = tenants.get(tenant_id) {
            // 检查租户状态
            if tenant.status == TenantStatus::Suspended {
                return Ok(false);
            }
            
            // 检查配额
            let isolation_settings = self.isolation_settings.read().unwrap();
            if isolation_settings.resource_quotas {
                let under_request_quota = tenant.usage.requests + requests <= tenant.quota.requests;
                let under_token_quota = tenant.usage.tokens + tokens <= tenant.quota.tokens;
                
                return Ok(under_request_quota && under_token_quota);
            }
            
            Ok(true)
        } else {
            Err("Tenant not found".to_string())
        }
    }

    // 重置使用计数（例如每月运行一次）
    pub fn reset_usage(&self) {
        let mut tenants = self.tenants.write().unwrap();
        
        for tenant in tenants.values_mut() {
            tenant.usage = ResourceUsage {
                requests: 0,
                tokens: 0,
            };
        }
    }
} 