use crate::services::compliance::{ComplianceReport, ComplianceService, ReportType, ReportStatus};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub tenant_id: String,
    pub report_type: ReportType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub report: ComplianceReport,
}

#[derive(Debug, Serialize)]
pub struct ReportsResponse {
    pub reports: Vec<ComplianceReport>,
}

pub async fn generate_report(
    State(compliance_service): State<Arc<Mutex<ComplianceService>>>,
    Json(request): Json<GenerateReportRequest>,
) -> impl IntoResponse {
    let mut service = compliance_service.lock().await;
    let report = service.generate_report(
        request.tenant_id,
        request.report_type,
        request.start_time,
        request.end_time,
    );

    (StatusCode::CREATED, Json(ReportResponse { report }))
}

pub async fn get_report(
    State(compliance_service): State<Arc<Mutex<ComplianceService>>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    let service = compliance_service.lock().await;
    match service.get_report(&report_id) {
        Some(report) => (StatusCode::OK, Json(ReportResponse { report: report.clone() })),
        None => (StatusCode::NOT_FOUND, Json(ReportResponse { report: Default::default() })),
    }
}

pub async fn get_tenant_reports(
    State(compliance_service): State<Arc<Mutex<ComplianceService>>>,
    Path(tenant_id): Path<String>,
) -> impl IntoResponse {
    let service = compliance_service.lock().await;
    let reports = service.get_tenant_reports(&tenant_id);
    (StatusCode::OK, Json(ReportsResponse { reports: reports.into_iter().cloned().collect() }))
}

pub async fn update_report_status(
    State(compliance_service): State<Arc<Mutex<ComplianceService>>>,
    Path(report_id): Path<String>,
    Json(status): Json<ReportStatus>,
) -> impl IntoResponse {
    let mut service = compliance_service.lock().await;
    match service.update_report_status(&report_id, status) {
        Some(report) => (StatusCode::OK, Json(ReportResponse { report: report.clone() })),
        None => (StatusCode::NOT_FOUND, Json(ReportResponse { report: Default::default() })),
    }
}

pub fn router(compliance_service: Arc<Mutex<ComplianceService>>) -> axum::Router {
    axum::Router::new()
        .route("/reports", axum::routing::post(generate_report))
        .route("/reports/:report_id", axum::routing::get(get_report))
        .route("/reports/tenant/:tenant_id", axum::routing::get(get_tenant_reports))
        .route("/reports/:report_id/status", axum::routing::put(update_report_status))
        .with_state(compliance_service)
} 