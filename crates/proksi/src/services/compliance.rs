use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub tenant_id: String,
    pub report_type: ReportType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub metrics: HashMap<String, f64>,
    pub violations: Vec<ComplianceViolation>,
    pub status: ReportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    DataPrivacy,
    Security,
    Performance,
    Cost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub id: String,
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

pub struct ComplianceService {
    reports: HashMap<String, ComplianceReport>,
}

impl ComplianceService {
    pub fn new() -> Self {
        Self {
            reports: HashMap::new(),
        }
    }

    pub fn generate_report(
        &mut self,
        tenant_id: String,
        report_type: ReportType,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> ComplianceReport {
        let report = ComplianceReport {
            id: uuid::Uuid::new_v4().to_string(),
            tenant_id,
            report_type,
            start_time,
            end_time,
            metrics: HashMap::new(),
            violations: Vec::new(),
            status: ReportStatus::Pending,
        };

        self.reports.insert(report.id.clone(), report.clone());
        report
    }

    pub fn get_report(&self, report_id: &str) -> Option<&ComplianceReport> {
        self.reports.get(report_id)
    }

    pub fn get_tenant_reports(&self, tenant_id: &str) -> Vec<&ComplianceReport> {
        self.reports
            .values()
            .filter(|report| report.tenant_id == tenant_id)
            .collect()
    }

    pub fn update_report_status(
        &mut self,
        report_id: &str,
        status: ReportStatus,
    ) -> Option<&ComplianceReport> {
        if let Some(report) = self.reports.get_mut(report_id) {
            report.status = status;
            Some(report)
        } else {
            None
        }
    }

    pub fn add_violation(
        &mut self,
        report_id: &str,
        rule_id: String,
        severity: ViolationSeverity,
        description: String,
    ) -> Option<&ComplianceReport> {
        if let Some(report) = self.reports.get_mut(report_id) {
            let violation = ComplianceViolation {
                id: uuid::Uuid::new_v4().to_string(),
                rule_id,
                severity,
                description,
                timestamp: Utc::now(),
                resolved: false,
            };
            report.violations.push(violation);
            Some(report)
        } else {
            None
        }
    }

    pub fn resolve_violation(
        &mut self,
        report_id: &str,
        violation_id: &str,
    ) -> Option<&ComplianceReport> {
        if let Some(report) = self.reports.get_mut(report_id) {
            if let Some(violation) = report
                .violations
                .iter_mut()
                .find(|v| v.id == violation_id)
            {
                violation.resolved = true;
            }
            Some(report)
        } else {
            None
        }
    }
} 