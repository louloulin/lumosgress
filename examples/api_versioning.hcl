// API Versioning Example Configuration
// This configuration demonstrates routing based on API version headers

service_name = "proksi"
worker_threads = 4

routes = [
  // Route for v1 of the API
  {
    host = "api.example.com"
    
    match_with {
      path = {
        patterns = ["/users/*"]
      }
      header = {
        name = "X-API-Version"
        values = ["1", "1.0"]
      }
    }

    upstreams = [
      {
        ip = "10.0.1.10"
        port = 8001
      }
    ]

    plugins = [
      { name = "request_id" }
    ]
    
    // Optionally, could add a version field (not implemented yet)
    // versions = ["1", "1.0"]
  },

  // Route for v2 of the API
  {
    host = "api.example.com"
    
    match_with {
      path = {
        patterns = ["/users/*", "/posts/*"] // v2 supports posts
      }
      header = {
        name = "X-API-Version"
        values = ["2", "2.0"]
      }
    }

    upstreams = [
      {
        ip = "10.0.2.20"
        port = 8002
      }
    ]

    plugins = [
      { name = "request_id" },
      // Add a plugin specific to v2
      // { name = "new_v2_feature_plugin" }
    ]
    
    // versions = ["2", "2.0"]
  },

  // Fallback route for requests without a version header or unmatched versions
  // (Could point to latest stable, or return an error/documentation)
  {
    host = "api.example.com"
    
    match_with {
      path = {
        patterns = ["/*"] // Match any path as a fallback
      }
      // No header matcher, or explicitly match missing/invalid header
    }

    upstreams = [
      {
        ip = "10.0.1.10" // Point to v1 by default
        port = 8001
      }
    ]
    
    plugins = [] 
  }
] 