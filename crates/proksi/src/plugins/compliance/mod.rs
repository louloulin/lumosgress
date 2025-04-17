use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::info;

use crate::{models::tenant::TenantInfo, plugins::{Plugin, PluginError}};

/// Configuration for the compliance plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Whether to enable compliance checks
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Retention policy in days
    #[serde(default = "default_retention")]
    pub retention_days: u32,
    
    /// Storage path for compliance logs
    #[serde(default = "default_storage_path")]
    pub storage_path: String,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            retention_days: default_retention(),
            storage_path: default_storage_path(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_retention() -> u32 {
    90
}

fn default_storage_path() -> String {
    "/var/log/proksi/compliance".to_string()
}

/// Record of a compliance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRecord {
    /// Unique ID for this record
    pub id: String,
    
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    
    /// API endpoint that was accessed
    pub endpoint: String,
    
    /// HTTP method used
    pub method: String,
    
    /// User or system that made the request
    pub requester: String,
    
    /// Tenant information
    pub tenant: Option<TenantInfo>,
    
    /// Whether the request was allowed
    pub allowed: bool,
    
    /// Reason for the decision
    pub reason: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Compliance Plugin for regulatory requirements
#[derive(Debug, Default)]
pub struct CompliancePlugin {
    config: ComplianceConfig,
    records: Vec<ComplianceRecord>,
}

#[async_trait]
impl Plugin for CompliancePlugin {
    fn name(&self) -> &'static str {
        "compliance"
    }
    
    async fn start(&mut self) -> Result<(), PluginError> {
        // Load any existing compliance records
        // This is a placeholder for actual implementation
        // We might need access to config here. If so, Default needs adjustment.
        info!("CompliancePlugin start. Config enabled: {}", self.config.enabled);
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), PluginError> {
        // Save any pending compliance records
        // This is a placeholder for actual implementation
        self.flush_records().map_err(|e| PluginError::StopFailed(e.to_string()))
    }
}

impl CompliancePlugin {
    /// Record a compliance event
    pub fn record(&mut self, record: ComplianceRecord) {
        if self.config.enabled {
            self.records.push(record);
            
            // Auto-flush if we have too many records
            if self.records.len() >= 100 {
                if let Err(e) = self.flush_records() {
                    tracing::error!("Failed to flush compliance records: {}", e);
                }
            }
        }
    }
    
    /// Flush records to storage
    fn flush_records(&mut self) -> Result<()> {
        if self.records.is_empty() {
            return Ok(());
        }
        
        // In a real implementation, we would write to disk or database here
        tracing::info!("Flushing {} compliance records", self.records.len());
        
        // Clear the records after successful flush
        self.records.clear();
        
        Ok(())
    }
} 