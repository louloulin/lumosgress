use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantStatus {
    Active,
    Suspended,
    Pending,
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