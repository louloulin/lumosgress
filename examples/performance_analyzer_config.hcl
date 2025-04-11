// Proksi Performance Analyzer Configuration Example

// Global settings
global {
  address = "0.0.0.0:8000"
  workers = 4
  tls_auto = true
}

// Route rules
router {
  // AI Gateway route
  route {
    name = "ai-gateway"
    host = "ai.example.com"
    
    // Match all paths
    match_with {
      path = {
        patterns = ["/*"]
      }
    }

    // Upstream servers
    upstreams = [
      {
        ip = "127.0.0.1"
        port = 3000
      }
    ]

    // Performance Analyzer plugin configuration
    plugins = [
      {
        name = "performance_analyzer"
        config = {
          // Enable the plugin
          enabled = true
          
          // Custom endpoint for the UI
          ui_endpoint = "/performance-dashboard"
          
          // Sample 25% of all requests to reduce overhead
          sample_rate = 0.25
          
          // Enable detailed profiling for more insights
          detailed_profiling = true
          
          // Enable automatic hot spot detection
          hotspot_detection = true
          
          // Track token usage for AI models
          profile_token_usage = true
          
          // Maximum trace depth for detailed profiling
          max_trace_depth = 15
          
          // Paths to exclude from monitoring
          exclude_paths = [
            "/health",
            "/metrics",
            "/favicon.ico",
            "/static/*"
          ]
          
          // Add trace headers to track requests across services
          trace_headers = true
          
          // Storage configuration
          storage = {
            // Choose storage type: memory, file, redis, or prometheus
            type = "memory"
            
            // Maximum entries to keep in memory
            max_entries = 50000
            
            // Uncomment for file storage
            // type = "file"
            // path = "/var/log/proksi/performance_metrics.json"
            
            // Uncomment for Redis storage
            // type = "redis"
            // url = "redis://redis:6379"
            // key_prefix = "proksi:perf:"
            
            // Uncomment for Prometheus storage
            // type = "prometheus"
            // endpoint = "/metrics"
          }
        }
      }
    ]
  }
} 