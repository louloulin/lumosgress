#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use http::StatusCode;
    use pingora::proxy::Session;
    use crate::proxy_server::https_proxy::RouterContext;
    use crate::proxy_server::HttpResponse;
    use crate::route::{RouteStore, RouteUpstream, RouteStoreContainer};
    use crate::plugins::core::{Plugin, PluginStep, PluginError};
    use super::{BasicAuth, BasicAuthConfig};

    // Helper to create a test RouterContext
    fn create_test_context() -> RouterContext {
        let route_store = RouteStore::default();
        let route_store_container = RouteStoreContainer {
            store: Arc::new(route_store),
            plugins: Vec::new(), // No actual plugins needed for this context
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
            request_id: "basic-auth-test-123".to_string(),
        }
    }

    // Helper to create BasicAuth plugin instance
    fn create_plugin() -> BasicAuth {
        let config = BasicAuthConfig {
            user: "testuser".to_string(),
            pass: "testpass".to_string(),
        };
        BasicAuth::new(config)
    }

    #[tokio::test]
    async fn test_basic_auth_valid_credentials() {
        let plugin = create_plugin();
        let mut session = Session::new_dummy();
        let mut ctx = create_test_context();

        let credentials = base64::encode_block(b"testuser:testpass");
        let auth_header_value = format!("Basic {}", credentials);
        session.req_header_mut().insert_header("Authorization", &auth_header_value).unwrap();

        let result = plugin.handle_request(PluginStep::Request, &mut session, &mut ctx).await;
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(!handled); // Should not handle (allow request through)
        assert!(response.is_none());
    }

    #[tokio::test]
    async fn test_basic_auth_invalid_credentials() {
        let plugin = create_plugin();
        let mut session = Session::new_dummy();
        let mut ctx = create_test_context();

        let credentials = base64::encode_block(b"testuser:wrongpass");
        let auth_header_value = format!("Basic {}", credentials);
        session.req_header_mut().insert_header("Authorization", &auth_header_value).unwrap();

        let result = plugin.handle_request(PluginStep::Request, &mut session, &mut ctx).await;
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(handled); // Should handle (block request)
        assert!(response.is_some());
        let response = response.unwrap();
        assert_eq!(response.status_code, StatusCode::UNAUTHORIZED);
        assert!(response.headers.contains_key(http::header::WWW_AUTHENTICATE));
    }

    #[tokio::test]
    async fn test_basic_auth_missing_header() {
        let plugin = create_plugin();
        let mut session = Session::new_dummy();
        let mut ctx = create_test_context();

        // No Authorization header added

        let result = plugin.handle_request(PluginStep::Request, &mut session, &mut ctx).await;
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(handled); // Should handle (block request)
        assert!(response.is_some());
        let response = response.unwrap();
        assert_eq!(response.status_code, StatusCode::UNAUTHORIZED);
        assert!(response.headers.contains_key(http::header::WWW_AUTHENTICATE));
    }

    #[tokio::test]
    async fn test_basic_auth_wrong_step() {
        let plugin = create_plugin();
        let mut session = Session::new_dummy();
        let mut ctx = create_test_context();

        let credentials = base64::encode_block(b"testuser:testpass");
        let auth_header_value = format!("Basic {}", credentials);
        session.req_header_mut().insert_header("Authorization", &auth_header_value).unwrap();

        // Test at a different step
        let result = plugin.handle_request(PluginStep::EarlyRequest, &mut session, &mut ctx).await;
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(!handled); // Should not handle at this step
        assert!(response.is_none());
    }
} 