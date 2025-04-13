use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::services::tenant::{IsolationSettings, ResourceQuota, ResourceUsage, Tenant, TenantService, TenantStatus};

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub status: TenantStatus,
    pub quota: ResourceQuota,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub tenant: Tenant,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl CreateTenantRequest {
    fn into_tenant(self) -> Tenant {
        Tenant {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            status: self.status,
            quota: self.quota,
            usage: Default::default(),
            plan: "basic".to_string(),
            users: Vec::new(),
        }
    }
}

pub fn tenant_routes(tenant_service: Arc<Mutex<TenantService>>) -> Router {
    Router::new()
        .route("/tenants", get(get_tenants))
        .route("/tenants", post(create_tenant))
        .route("/tenants/:id", get(get_tenant))
        .route("/tenants/:id", put(update_tenant))
        .route("/tenants/:id/toggle-status", post(toggle_tenant_status))
        .route("/tenants/isolation-settings", get(get_isolation_settings))
        .route("/tenants/isolation-settings", put(update_isolation_settings))
        .with_state(tenant_service)
}

async fn get_tenants(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
) -> impl IntoResponse {
    let service = tenant_service.lock().await;
    let tenants = service.get_all_tenants();
    (StatusCode::OK, Json(tenants))
}

async fn get_tenant(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
    Path(id): Path<String>,
) -> Response {
    let service = tenant_service.lock().await;
    match service.get_tenant(&id) {
        Some(tenant) => (StatusCode::OK, Json(TenantResponse { tenant: tenant.clone() })).into_response(),
        None => (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Tenant not found".to_string() })).into_response(),
    }
}

async fn create_tenant(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
    Json(tenant_data): Json<CreateTenantRequest>,
) -> Response {
    let mut service = tenant_service.lock().await;
    let tenant = tenant_data.into_tenant();
    match service.create_tenant(tenant) {
        Ok(tenant) => (StatusCode::CREATED, Json(TenantResponse { tenant: tenant.clone() })).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response(),
    }
}

async fn update_tenant(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
    Path(id): Path<String>,
    Json(tenant_data): Json<CreateTenantRequest>,
) -> Response {
    let mut service = tenant_service.lock().await;
    let tenant = tenant_data.into_tenant();
    match service.update_tenant(&id, tenant) {
        Ok(tenant) => (StatusCode::OK, Json(TenantResponse { tenant: tenant.clone() })).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(ErrorResponse { error: e })).into_response(),
    }
}

async fn toggle_tenant_status(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
    Path(id): Path<String>,
) -> Response {
    let mut service = tenant_service.lock().await;
    match service.toggle_tenant_status(&id) {
        Ok(tenant) => (StatusCode::OK, Json(TenantResponse { tenant: tenant.clone() })).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(ErrorResponse { error: e })).into_response(),
    }
}

async fn get_isolation_settings(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
) -> impl IntoResponse {
    let service = tenant_service.lock().await;
    let settings = service.get_isolation_settings();
    (StatusCode::OK, Json(settings))
}

async fn update_isolation_settings(
    State(tenant_service): State<Arc<Mutex<TenantService>>>,
    Json(settings): Json<IsolationSettings>,
) -> impl IntoResponse {
    let mut service = tenant_service.lock().await;
    let updated_settings = service.update_isolation_settings(settings);
    (StatusCode::OK, Json(updated_settings))
} 