// WebSocket Routing Example Configuration
// This configuration demonstrates how to route WebSocket traffic

service_name = "proksi"
worker_threads = 4

routes = [
  {
    host = "app.example.com"
    
    // Match both standard HTTP/HTTPS and WebSocket traffic
    match_with {
      path = {
        patterns = ["/*"]
      }
    }

    // Upstream server that handles both HTTP and WebSocket connections
    upstreams = [
      {
        ip = "192.168.1.100"
        port = 8080
      }
    ]

    // Plugins can still be applied, but ensure they are WebSocket-compatible
    // For example, authentication or request ID plugins might work,
    // but caching or complex body transformations would likely be skipped
    // due to the logic added in https_proxy.rs.
    plugins = [
      {
        name = "request_id"
        config = {
          header_name = "X-Request-ID"
        }
      },
      // {
      //   name = "my_websocket_auth_plugin" 
      // }
    ]
  },
  {
    host = "chat.example.com"
    
    // Specific path for WebSocket connections
    match_with {
      path = {
        patterns = ["/ws"]
      }
    }

    // Dedicated upstream for WebSocket connections
    upstreams = [
      {
        ip = "192.168.1.101"
        port = 9000
      }
    ]
    
    // No plugins for this simple WebSocket route
    plugins = []
  }
] 