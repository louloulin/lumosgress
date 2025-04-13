mod tenant_api;

use std::sync::Arc;
use axum::Router;
use crate::services::tenant::TenantService;
use tenant_api::tenant_routes;
use tokio::sync::Mutex;

pub fn setup_api_routes() -> Router {
    let tenant_service = Arc::new(Mutex::new(TenantService::new()));
    
    let router = Router::new()
        .nest("/api", tenant_routes(tenant_service.clone()));
    
    router
} 