pub mod api;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use tokio::net::TcpListener;
use tracing::{debug, info};

use crate::config::Config;

/// 启动 API 服务器
pub async fn start_api_server(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 API 路由
    let app = api::setup_api_routes();
    
    // 设置监听地址（默认为 localhost:5173，与前端开发服务器相同）
    let addr = SocketAddr::from(([127, 0, 0, 1], 5173));
    
    // 创建 TCP 监听器
    let listener = TcpListener::bind(&addr).await?;
    debug!("API server listening on {}", addr);
    
    // 启动 API 服务器
    info!("API server started on http://{}", addr);
    axum::serve(listener, app).await?;
    
    Ok(())
}
