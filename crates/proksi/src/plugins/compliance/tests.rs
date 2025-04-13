#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use pingora::proxy::Session;
    use crate::config::RoutePlugin;
    use crate::proxy_server::https_proxy::RouterContext;
    use crate::plugins::compliance::CompliancePlugin;
    use crate::plugins::MiddlewarePlugin;
    use std::borrow::Cow;
    use serde_json::json;

    #[tokio::test]
    async fn test_compliance_plugin_config() {
        let compliance_plugin = CompliancePlugin::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/api".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: crate::proxy_server::https_proxy::RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        // Create a test plugin configuration
        let mut config_map = HashMap::new();
        config_map.insert(Cow::Borrowed("retention_period_days"), json!(180));
        config_map.insert(Cow::Borrowed("alert_threshold"), json!(0.75));
        
        let plugin = RoutePlugin {
            name: Cow::Borrowed("compliance"),
            config: Some(config_map),
        };
        
        // Test request filter
        let result = compliance_plugin.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
    }
} 