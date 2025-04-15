#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use crate::plugins::api_server::{ApiServerConfig, ApiServerPlugin};
    use crate::plugins::Plugin;
    use crate::config::Config;

    #[tokio::test]
    async fn test_api_server_config() {
        let config = ApiServerConfig {
            listen_addr: "127.0.0.1:0".to_string(), // Use port 0 for auto-assignment in tests
            access_log: true,
            cors: true,
        };

        assert_eq!(config.listen_addr, "127.0.0.1:0");
        assert_eq!(config.access_log, true);
        assert_eq!(config.cors, true);
    }

    #[tokio::test]
    async fn test_api_server_create() {
        let config = ApiServerConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            access_log: false,
            cors: false,
        };

        let plugin = ApiServerPlugin::new(config).await.unwrap();
        assert_eq!(plugin.name(), "api_server");
    }

    #[tokio::test]
    async fn test_api_server_with_system_config() {
        let config = ApiServerConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            access_log: true,
            cors: true,
        };

        let system_config = Arc::new(Config::default());
        let plugin = ApiServerPlugin::new(config).await.unwrap();
        let plugin = plugin.with_system_config(system_config.clone());

        assert_eq!(plugin.name(), "api_server");
    }

    #[tokio::test]
    async fn test_api_server_start_stop() {
        let config = ApiServerConfig {
            listen_addr: "127.0.0.1:0".to_string(), // Use port 0 for auto-assignment
            access_log: false,
            cors: false,
        };

        let system_config = Arc::new(Config::default());
        let mut plugin = ApiServerPlugin::new(config).await.unwrap();
        plugin = plugin.with_system_config(system_config);

        // Start the server
        plugin.start().await.unwrap();
        assert!(plugin.server_handle.is_some());
        assert!(plugin.shutdown_tx.is_some());

        // Give it a moment to start up
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop the server
        plugin.stop().await.unwrap();
        assert!(plugin.server_handle.is_none());
        assert!(plugin.shutdown_tx.is_none());
    }
}