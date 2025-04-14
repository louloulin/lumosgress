#[cfg(test)]
mod tests {
    use std::sync::Arc;
    
    use async_trait::async_trait;
    use pingora::http::ResponseHeader;
    use pingora::proxy::Session;
    
    use crate::plugins::core::{Plugin, PluginStep};
    use crate::proxy_server::https_proxy::RouterContext;
    use crate::proxy_server::router::RouteStoreContainer;
    use crate::route::{RouteStore, RouteUpstream};
    
    use super::{RequestId, RequestIdConfig};
    
    // 创建测试用的RouterContext
    fn create_test_context() -> RouterContext {
        let route_store = RouteStore::default();
        let route_store_container = RouteStoreContainer {
            store: Arc::new(route_store),
            plugins: Vec::new(),
        };
        
        let upstream = RouteUpstream::default();
        
        RouterContext {
            host: "test.example.com".to_string(),
            route_container: route_store_container,
            upstream,
            extensions: std::collections::HashMap::new(),
            is_websocket: false,
            timings: Default::default(),
            upstream_response: None,
            plugins_data: std::collections::HashMap::new(),
            request_id: String::new(),
        }
    }
    
    #[tokio::test]
    async fn test_request_id_generation() {
        let plugin = RequestId::new();
        let mut ctx = create_test_context();
        let mut session = Session::new_dummy();
        
        // 测试 handle_request
        let result = plugin.handle_request(
            PluginStep::Request,
            &mut session,
            &mut ctx
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(!handled); // 不应该中断处理链
        assert!(response.is_none()); // 不应该返回响应
        
        // 验证 request_id 已存储
        assert!(!ctx.request_id.is_empty());
        assert!(ctx.extensions.contains_key("request_id_header"));
        assert_eq!(
            ctx.extensions.get("request_id_header").unwrap(), 
            &ctx.request_id
        );
    }
    
    #[tokio::test]
    async fn test_response_header_with_request_id() {
        let plugin = RequestId::with_config(RequestIdConfig {
            enabled: true,
            header_name: "x-test-request-id".to_string(),
        });
        
        let mut ctx = create_test_context();
        let mut session = Session::new_dummy();
        
        // 首先生成一个请求ID
        plugin.handle_request(
            PluginStep::Request,
            &mut session,
            &mut ctx
        ).await.unwrap();
        
        // 验证请求ID不为空
        assert!(!ctx.request_id.is_empty());
        
        // 创建响应头
        let mut response_header = ResponseHeader::build(200, None).unwrap();
        
        // 处理响应
        let result = plugin.handle_response(
            PluginStep::Response,
            &mut session,
            &mut ctx,
            &mut response_header
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        assert!(result.unwrap()); // 表示已修改响应
        
        // 验证响应头包含请求ID
        let header_value = response_header.get_header("x-test-request-id").unwrap();
        assert_eq!(header_value, ctx.request_id);
    }
    
    #[tokio::test]
    async fn test_disabled_plugin() {
        let plugin = RequestId::with_config(RequestIdConfig {
            enabled: false,
            header_name: "x-request-id".to_string(),
        });
        
        let mut ctx = create_test_context();
        let mut session = Session::new_dummy();
        
        // 处理请求
        let result = plugin.handle_request(
            PluginStep::Request,
            &mut session,
            &mut ctx
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(!handled);
        assert!(response.is_none());
        
        // 验证请求ID为空（因为插件被禁用）
        assert!(ctx.request_id.is_empty());
        assert!(!ctx.extensions.contains_key("request_id_header"));
        
        // 创建响应头
        let mut response_header = ResponseHeader::build(200, None).unwrap();
        
        // 处理响应
        let result = plugin.handle_response(
            PluginStep::Response,
            &mut session,
            &mut ctx,
            &mut response_header
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 表示未修改响应
        
        // 验证响应头不包含请求ID
        assert!(response_header.get_header("x-request-id").is_none());
    }
} 