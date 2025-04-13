use serde::{Deserialize, Serialize};
use std::sync::Arc;
use async_trait::async_trait;
use crate::plugins::{Plugin, PluginConfig, PluginError};
use crate::models::tenant::{Tenant, TenantStatus, ResourceQuota, ResourceUsage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPluginConfig {
    pub default_quota: ResourceQuota,
    pub isolation_enabled: bool,
}

impl PluginConfig for TenantPluginConfig {}

pub struct TenantPlugin {
    config: Arc<TenantPluginConfig>,
    tenants: dashmap::DashMap<String, Tenant>,
}

#[async_trait]
impl Plugin for TenantPlugin {
    type Config = TenantPluginConfig;

    async fn new(config: Self::Config) -> Result<Self, PluginError> {
        Ok(Self {
            config: Arc::new(config),
            tenants: dashmap::DashMap::new(),
        })
    }

    fn name(&self) -> &'static str {
        "tenant"
    }
}

impl TenantPlugin {
    pub async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, PluginError> {
        if self.tenants.contains_key(&tenant.id) {
            return Err(PluginError::AlreadyExists("Tenant already exists"));
        }
        
        self.tenants.insert(tenant.id.clone(), tenant.clone());
        Ok(tenant)
    }

    pub async fn get_tenant(&self, id: &str) -> Option<Tenant> {
        self.tenants.get(id).map(|t| t.value().clone())
    }

    // 其他租户操作方法...
} 