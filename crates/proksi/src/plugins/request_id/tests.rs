#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use futures_util::FutureExt;
    use pingora::http::{ResponseHeader, StatusCode};
    use pingora::proxy::Session;
    use tokio_test::io::Builder;
    
    use crate::plugins::core::{Plugin, PluginStep};
    use crate::plugins::request_id::{RequestId, RequestIdConfig};
    use crate::proxy_server::https_proxy::RouterContext;
    
    // 创建测试用的RouterContext
    fn create_test_context() -> RouterContext {
        RouterContext {
            host: "test.example.com".to_string(),
            route_container: Default::default(),
            upstream: Default::default(),
            extensions: HashMap::new(),
            is_websocket: false,
            timings: Default::default(),
            plugins_data: HashMap::new(),
            request_id: String::new(),
            upstream_response: None,
        }
    }
    
    fn create_test_session(path: &[u8]) -> Session {
        let headers = [""].join("\r\n");
        let input_header = format!(
            "GET {} HTTP/1.1\r\n{headers}\r\n\r\n",
            String::from_utf8_lossy(path)
        );
        let mock_io = Builder::new().read(input_header.as_bytes()).build();
        let mut session = Session::new_h1(Box::new(mock_io));
        session.read_request().now_or_never().unwrap().unwrap();
        session
    }
    
    #[tokio::test]
    async fn test_request_id_generation() {
        let plugin = RequestId::new();
        let mut ctx = create_test_context();
        let mut session = create_test_session(b"/test");
        
        // 测试 handle_request
        let result = plugin.handle_request(
            PluginStep::EarlyRequest,
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
        let plugin = RequestId::new();
        let mut ctx = create_test_context();
        let mut session = create_test_session(b"/test");
        
        // 首先生成一个请求ID
        let result = plugin.handle_request(
            PluginStep::EarlyRequest,
            &mut session,
            &mut ctx
        ).await;
        assert!(result.is_ok());
        
        // 创建响应头
        let mut response_header = ResponseHeader::build(StatusCode::OK, None).unwrap();
        
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
        assert!(response_header.headers.contains_key("x-request-id"));
        assert_eq!(
            response_header.headers.get("x-request-id").unwrap().to_str().unwrap(),
            ctx.request_id
        );
    }
    
    #[tokio::test]
    async fn test_disabled_plugin() {
        let plugin = RequestId::with_config(RequestIdConfig {
            enabled: false,
            header_name: "x-request-id".to_string(),
            preserve_existing: true,
            add_to_upstream: false,
        });
        
        let mut ctx = create_test_context();
        let mut session = create_test_session(b"/test");
        
        // 处理请求
        let result = plugin.handle_request(
            PluginStep::EarlyRequest,
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
        let mut response_header = ResponseHeader::build(StatusCode::OK, None).unwrap();
        
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
        assert!(!response_header.headers.contains_key("x-request-id"));
    }
}