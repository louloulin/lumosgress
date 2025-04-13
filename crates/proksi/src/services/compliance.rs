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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_generate_report() {
        let mut service = ComplianceService::new();
        let start_time = Utc::now();
        let end_time = start_time + Duration::days(7);

        let report = service.generate_report(
            "tenant1".to_string(),
            ReportType::DataPrivacy,
            start_time,
            end_time,
        );

        assert_eq!(report.tenant_id, "tenant1");
        assert_eq!(report.report_type, ReportType::DataPrivacy);
        assert_eq!(report.status, ReportStatus::Pending);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_add_violation() {
        let mut service = ComplianceService::new();
        let start_time = Utc::now();
        let end_time = start_time + Duration::days(7);

        let report = service.generate_report(
            "tenant1".to_string(),
            ReportType::DataPrivacy,
            start_time,
            end_time,
        );

        let updated_report = service.add_violation(
            &report.id,
            "rule1".to_string(),
            ViolationSeverity::High,
            "Test violation".to_string(),
        );

        assert!(updated_report.is_some());
        let report = updated_report.unwrap();
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].severity, ViolationSeverity::High);
        assert_eq!(report.violations[0].description, "Test violation");
    }

    #[test]
    fn test_update_report_status() {
        let mut service = ComplianceService::new();
        let start_time = Utc::now();
        let end_time = start_time + Duration::days(7);

        let report = service.generate_report(
            "tenant1".to_string(),
            ReportType::DataPrivacy,
            start_time,
            end_time,
        );

        let updated_report = service.update_report_status(&report.id, ReportStatus::Completed);
        assert!(updated_report.is_some());
        assert_eq!(updated_report.unwrap().status, ReportStatus::Completed);
    }

    #[test]
    fn test_get_tenant_reports() {
        let mut service = ComplianceService::new();
        let start_time = Utc::now();
        let end_time = start_time + Duration::days(7);

        service.generate_report(
            "tenant1".to_string(),
            ReportType::DataPrivacy,
            start_time,
            end_time,
        );

        service.generate_report(
            "tenant1".to_string(),
            ReportType::Security,
            start_time,
            end_time,
        );

        service.generate_report(
            "tenant2".to_string(),
            ReportType::DataPrivacy,
            start_time,
            end_time,
        );

        let tenant1_reports = service.get_tenant_reports("tenant1");
        assert_eq!(tenant1_reports.len(), 2);

        let tenant2_reports = service.get_tenant_reports("tenant2");
        assert_eq!(tenant2_reports.len(), 1);
    }
} 