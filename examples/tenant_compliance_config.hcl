// Proksi Tenant and Compliance Configuration Example

// Global settings
service_name = "proksi"

// Global tenant configuration
tenant {
  enabled = true
  id_header = "x-tenant-id"
  default_quota {
    requests = 10000
    tokens = 500000
  }
}

// Global compliance configuration
compliance {
  enabled = true
  retention_days = 90
  alert_threshold = 0.85
}

// Route rules with tenant and compliance plugins
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

    // Tenant and compliance plugins configuration
    plugins = [
      {
        name = "tenant"
        config = {
          // Override global settings for this route if needed
          isolation_enabled = true
          default_quota = {
            requests = 20000
            tokens = 1000000
          }
        }
      },
      {
        name = "compliance"
        config = {
          // Override global settings for this route if needed
          retention_period_days = 180
          alert_threshold = 0.75
        }
      }
    ]
  }
] 