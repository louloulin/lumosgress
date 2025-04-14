#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use async_trait::async_trait;
    use pingora::http::ResponseHeader;
    use pingora::proxy::Session;
    use serde_json::json;
    
    use crate::plugins::{Plugin, PluginError, PluginStep};
    use crate::proxy_server::https_proxy::RouterContext;
    use crate::proxy_server::router::RouteStoreContainer;
    use crate::route::{RouteStore, RouteUpstream};
    
    // 测试插件，用于测试RouterContext中的plugins_data和upstream_response字段
    struct TestContextPlugin {
        name: &'static str,
    }
    
    impl TestContextPlugin {
        fn new(name: &'static str) -> Self {
            Self { name }
        }
    }
    
    #[async_trait]
    impl Plugin for TestContextPlugin {
        fn name(&self) -> &'static str {
            self.name
        }
        
        async fn handle_request(
            &self,
            step: PluginStep,
            _session: &mut Session,
            ctx: &mut RouterContext,
        ) -> Result<(bool, Option<pingora::http::HttpResponse>), PluginError> {
            // 在插件数据中存储一些信息
            ctx.plugins_data.insert(
                self.name.to_string(),
                json!({
                    "step": format!("{:?}", step),
                    "processed": true,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            );
            
            Ok((false, None))
        }
        
        async fn handle_response(
            &self,
            step: PluginStep,
            _session: &mut Session,
            ctx: &mut RouterContext,
            upstream_response: &mut ResponseHeader,
        ) -> Result<bool, PluginError> {
            // 在插件数据中存储上游响应的状态码
            if let Some(status) = upstream_response.status() {
                ctx.plugins_data.insert(
                    format!("{}_response", self.name),
                    json!({
                        "step": format!("{:?}", step),
                        "status_code": status.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }),
                );
            }
            
            // 修改响应头
            upstream_response.append_header("X-Processed-By", self.name)?;
            
            Ok(true)
        }
        
        async fn start(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
        
        async fn stop(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }
    
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
            extensions: HashMap::new(),
            is_websocket: false,
            timings: Default::default(),
            upstream_response: None,
            plugins_data: HashMap::new(),
            request_id: "test-request-123".to_string(),
        }
    }
    
    #[tokio::test]
    async fn test_plugins_data_field() {
        let mut ctx = create_test_context();
        let mut session = Session::new_dummy();
        
        let plugin = TestContextPlugin::new("test_plugin");
        
        // 执行请求处理
        let result = plugin.handle_request(
            PluginStep::Request, 
            &mut session, 
            &mut ctx
        ).await;
        
        assert!(result.is_ok());
        let (handled, _) = result.unwrap();
        assert!(!handled);
        
        // 验证插件数据已被正确存储
        assert!(ctx.plugins_data.contains_key("test_plugin"));
        let data = &ctx.plugins_data["test_plugin"];
        assert_eq!(data["step"], "Request");
        assert_eq!(data["processed"], true);
    }
    
    #[tokio::test]
    async fn test_upstream_response_field() {
        let mut ctx = create_test_context();
        let mut session = Session::new_dummy();
        
        // 创建一个测试响应头
        let mut response_header = ResponseHeader::build(200, None).unwrap();
        response_header.append_header("Content-Type", "application/json").unwrap();
        
        // 设置上游响应
        ctx.upstream_response = Some(response_header.clone());
        
        let plugin = TestContextPlugin::new("test_plugin");
        
        // 执行响应处理
        let result = plugin.handle_response(
            PluginStep::Response, 
            &mut session, 
            &mut ctx,
            &mut response_header
        ).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // 验证插件数据已被正确存储
        let key = "test_plugin_response";
        assert!(ctx.plugins_data.contains_key(key));
        let data = &ctx.plugins_data[key];
        assert_eq!(data["step"], "Response");
        assert_eq!(data["status_code"], "200");
        
        // 验证响应头被修改
        let processed_by = response_header.get_header("X-Processed-By").unwrap();
        assert_eq!(processed_by, "test_plugin");
    }
    
    #[tokio::test]
    async fn test_request_id_field() {
        let mut ctx = create_test_context();
        
        // 验证请求ID已被设置
        assert_eq!(ctx.request_id, "test-request-123");
        
        // 修改请求ID
        ctx.request_id = "new-request-456".to_string();
        
        // 验证请求ID已被修改
        assert_eq!(ctx.request_id, "new-request-456");
    }
} 