use serde::{Deserialize, Serialize};
use std::sync::Arc;
use async_trait::async_trait;
use crate::plugins::{Plugin, PluginError};
use crate::models::tenant::{Tenant, TenantStatus, ResourceQuota, ResourceUsage};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TenantPluginConfig {
    pub default_quota: ResourceQuota,
    pub isolation_enabled: bool,
}

impl Default for TenantPluginConfig {
    fn default() -> Self {
        Self {
            default_quota: ResourceQuota {
                requests: 1000,
                tokens: 1000000,
            },
            isolation_enabled: false,
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct TenantPlugin {
    config: Arc<TenantPluginConfig>,
    tenants: dashmap::DashMap<String, Tenant>,
}

#[async_trait]
impl Plugin for TenantPlugin {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tenant::ResourceQuota;

    #[test]
    fn test_tenant_plugin_default_config() {
        let config = TenantPluginConfig::default();
        assert_eq!(config.default_quota.requests, 1000);
        assert_eq!(config.default_quota.tokens, 1000000);
        assert_eq!(config.isolation_enabled, false);
    }
} 