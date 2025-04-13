// Proksi Monitoring and Alerting Configuration Example

// Global settings
service_name = "proksi"
worker_threads = 4

// Monitoring configuration
monitoring {
  // Enable monitoring and alerting
  enabled = true
  
  // Alert threshold (default: 0.85)
  alert_threshold = 0.80
  
  // Data retention in days (default: 30)
  retention_days = 45
  
  // Enable anomaly detection (default: true)
  anomaly_detection = true
  
  // Sample rate for metrics collection (0.0-1.0)
  // Set to 0.5 to sample 50% of requests
  sample_rate = 1.0
  
  // Custom API endpoints
  dashboard_endpoint = "/monitoring-dashboard"
  alerts_endpoint = "/monitoring-alerts"
}

// Route configuration with monitoring
routes = [
  {
    host = "api.example.com"
    
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
    
    // Configure plugins for the route
    plugins = [
      {
        name = "performance_analyzer"
        config = {
          // Enable detailed profiling
          detailed_profiling = true
          
          // Sample rate for this specific route
          sample_rate = 1.0
          
          // Track token usage
          profile_token_usage = true
          
          // Custom endpoint
          ui_endpoint = "/performance"
        }
      },
      {
        name = "ai_analytics"
        config = {
          // Enable analytics
          enabled = true
          
          // Custom threshold for latency
          alert_thresholds = {
            latency_ms = 3000
            error_rate = 0.05
          }
          
          // Custom endpoint
          ui_endpoint = "/analytics"
          
          // Enable anomaly detection
          anomaly_detection = true
        }
      }
    ]
  }
] 