// API Server Configuration Example
// This example demonstrates how to configure the API Server plugin for monitoring and management

service "proksi" {
  // Global configuration
  service_name = "proksi"
  worker_threads = 4
  
  // Enable API server plugin
  plugins {
    api_server {
      // Address to listen on
      listen_address = "127.0.0.1:8080"
      
      // Enable access logging (default: true)
      enable_access_log = true
      
      // Enable CORS (default: true)
      enable_cors = true
    }
  }
  
  // Define routes for the main proxy service
  route {
    host = "api.example.com"
    path = "/"
    
    upstream {
      service = "api-backend"
      host = "backend.example.com"
      port = 8443
      tls = true
    }
    
    // Optional: Add performance analysis plugin to collect metrics
    // that will be accessible via the API server
    plugins {
      performance_analyzer {
        enabled = true
        sample_rate = 1.0
      }
    }
  }
}

// Server will expose the following endpoints:
// - http://127.0.0.1:8080/health - Health check
// - http://127.0.0.1:8080/metrics - Metrics data
// - http://127.0.0.1:8080/api/v1/config - System configuration 