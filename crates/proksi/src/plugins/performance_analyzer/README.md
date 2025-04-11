# Performance Analyzer Plugin

The Performance Analyzer plugin provides detailed performance metrics and analysis for AI API requests. It helps developers identify bottlenecks, optimize response times, and monitor system performance.

## Features

- **Request Performance Metrics**: Track processing times, latencies, and resource usage
- **Real-time Monitoring**: Monitor AI API request performance in real-time
- **Hot Spot Detection**: Automatically identify performance bottlenecks
- **Token Usage Profiling**: Track token usage for cost optimization
- **Flexible Storage Backends**: Store metrics in memory, files, Redis, or Prometheus
- **Performance Dashboard**: Visual representation of performance data
- **Configurable Sampling**: Control metric collection overhead

## Configuration

Add the plugin to your Proksi route configuration:

```hcl
plugins = [
  {
    name = "performance_analyzer"
    config = {
      // Enable or disable the plugin (default is true)
      enabled = true
      
      // Custom endpoint for the UI (default is /performance-analyzer)
      ui_endpoint = "/perf-dashboard"
      
      // Sampling rate between 0.0 and 1.0 (default is 1.0 = 100%)
      sample_rate = 0.1
      
      // Enable detailed profiling - more metrics but higher overhead (default is false)
      detailed_profiling = false
      
      // Enable hot spot detection (default is true)
      hotspot_detection = true
      
      // Profile token usage for AI requests (default is true)
      profile_token_usage = true
      
      // Maximum trace depth for detailed profiling (default is 10)
      max_trace_depth = 10
      
      // Paths to exclude from profiling
      exclude_paths = ["/health", "/metrics", "/favicon.ico"]
      
      // Whether to add trace headers to requests (default is false)
      trace_headers = false
      
      // Storage configuration for metrics
      storage = {
        // Storage type: "memory", "file", "redis", or "prometheus"
        type = "memory"
        
        // Maximum entries to keep in memory (for memory storage)
        max_entries = 10000
        
        // For file storage: path to save metrics
        // path = "./metrics.json"
        
        // For Redis storage: connection URL and key prefix
        // url = "redis://localhost:6379"
        // key_prefix = "perf:"
        
        // For Prometheus storage: endpoint to expose metrics
        // endpoint = "/metrics"
      }
    }
  }
]
```

## Metrics Dashboard

The Performance Analyzer provides a web-based dashboard at the configured endpoint (default: `/performance-analyzer`). The dashboard includes:

1. **Overview**: Key performance indicators like average response time and success rate
2. **Hot Spots**: Automatically detected performance bottlenecks with recommendations
3. **Request History**: Recent request details with filtering and sorting
4. **Charts**: Visual representation of performance trends
5. **Token Usage**: Token consumption analysis for cost optimization
6. **Provider Comparison**: Performance comparison across different LLM providers

## Performance Hot Spots

The plugin automatically detects performance issues such as:

- Slow upstream responses
- High token usage
- Request processing bottlenecks
- Frequently failing requests
- Unusually long total request times

Each hot spot includes:
- Impact level (Low, Medium, High, Critical)
- Description of the issue
- Average time affected
- Occurrence count
- Recommendations for improvement

## Storage Options

The plugin supports multiple storage backends:

1. **Memory**: In-memory storage with configurable size limit (default)
2. **File**: Persistent storage in a JSON file
3. **Redis**: High-performance storage in Redis
4. **Prometheus**: Metrics exposure in Prometheus format

## Best Practices

1. **Sampling Rate**: For high-traffic systems, reduce the sampling rate to minimize overhead
2. **Exclude Health Checks**: Always exclude health check endpoints from profiling
3. **Storage Selection**: Use memory storage for development and Redis/Prometheus for production
4. **Regular Review**: Check the dashboard regularly to identify new performance issues
5. **Progressive Enhancement**: Start with basic settings and enable detailed profiling as needed

## Example Usage

```hcl
plugins = [
  {
    name = "performance_analyzer"
    config = {
      sample_rate = 0.25
      ui_endpoint = "/performance"
      storage = {
        type = "redis"
        url = "redis://redis-server:6379"
        key_prefix = "ai-perf:"
      }
      exclude_paths = ["/health", "/metrics", "/static/*"]
    }
  }
] 