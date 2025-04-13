use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::tenant::{IsolationSettings, ResourceQuota, ResourceUsage, Tenant, TenantService, TenantStatus};

pub fn tenant_routes(tenant_service: Arc<TenantService>) -> Router {
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
    State(tenant_service): State<Arc<TenantService>>,
) -> impl IntoResponse {
    let tenants = tenant_service.get_all_tenants();
    (StatusCode::OK, Json(tenants))
}

async fn get_tenant(
    State(tenant_service): State<Arc<TenantService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match tenant_service.get_tenant(&id) {
        Some(tenant) => (StatusCode::OK, Json(tenant)),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Tenant not found" }))),
    }
}

async fn create_tenant(
    State(tenant_service): State<Arc<TenantService>>,
    Json(mut tenant_data): Json<Tenant>,
) -> impl IntoResponse {
    // 生成唯一ID (如果没有提供)
    if tenant_data.id.is_empty() {
        tenant_data.id = Uuid::new_v4().to_string();
    }
    
    // 确保使用量初始化为0
    tenant_data.usage = ResourceUsage {
        requests: 0,
        tokens: 0,
    };
    
    match tenant_service.create_tenant(tenant_data) {
        Ok(tenant) => (StatusCode::CREATED, Json(tenant)),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))),
    }
}

async fn update_tenant(
    State(tenant_service): State<Arc<TenantService>>,
    Path(id): Path<String>,
    Json(tenant_data): Json<Tenant>,
) -> impl IntoResponse {
    match tenant_service.update_tenant(&id, tenant_data) {
        Ok(tenant) => (StatusCode::OK, Json(tenant)),
        Err(e) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e }))),
    }
}

async fn toggle_tenant_status(
    State(tenant_service): State<Arc<TenantService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match tenant_service.toggle_tenant_status(&id) {
        Ok(tenant) => (StatusCode::OK, Json(tenant)),
        Err(e) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e }))),
    }
}

async fn get_isolation_settings(
    State(tenant_service): State<Arc<TenantService>>,
) -> impl IntoResponse {
    let settings = tenant_service.get_isolation_settings();
    (StatusCode::OK, Json(settings))
}

async fn update_isolation_settings(
    State(tenant_service): State<Arc<TenantService>>,
    Json(settings): Json<IsolationSettings>,
) -> impl IntoResponse {
    let updated_settings = tenant_service.update_isolation_settings(settings);
    (StatusCode::OK, Json(updated_settings))
} 