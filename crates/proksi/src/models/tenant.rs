use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 资源配额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub requests: u64,
    pub tokens: u64,
}

// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub requests: u64,
    pub tokens: u64,
}

// 租户状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    /// Tenant is active and fully operational
    Active,
    
    /// Tenant is inactive but can be reactivated
    Inactive,
    
    /// Tenant has been suspended due to policy violations
    Suspended,
    
    /// Tenant is in trial period
    Trial,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Active
    }
}

fn default_status() -> TenantStatus {
    TenantStatus::Active
}

// 租户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub plan: String, // e.g., "developer", "business", "enterprise"
    pub status: TenantStatus,
    pub users_count: u32,
    pub quota: ResourceQuota,
    pub usage: ResourceUsage,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents tenant information in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Unique identifier for the tenant
    pub id: String,
    
    /// Display name for the tenant
    pub name: String,
    
    /// Tenant status (active, suspended, etc.)
    #[serde(default = "default_status")]
    pub status: TenantStatus,
    
    /// Creation timestamp (ISO format)
    pub created_at: String,
    
    /// Custom attributes for the tenant
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

impl TenantInfo {
    /// Create a new tenant with the given ID and name
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            name: name.into(),
            status: TenantStatus::Active,
            created_at: now,
            attributes: HashMap::new(),
        }
    }
    
    /// Check if the tenant is active
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active || self.status == TenantStatus::Trial
    }
    
    /// Add an attribute to the tenant
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
} 