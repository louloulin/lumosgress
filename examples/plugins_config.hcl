// 插件系统配置示例
// 此示例展示如何配置Proksi的插件系统

service "proksi" {
  // 全局配置
  service_name = "proksi"
  worker_threads = 4
  
  // 插件配置
  plugins {
    // API服务器插件配置
    api_server {
      enabled = true
      listen_address = "127.0.0.1:8080"
      enable_access_log = true
      enable_cors = true
    }
    
    // 租户插件配置
    tenant {
      enabled = true
      default_requests = 5000    // 默认请求配额
      default_tokens = 50000     // 默认令牌配额
      isolation_enabled = true   // 是否启用租户隔离
    }
    
    // 合规插件配置
    compliance {
      enabled = true
      retention_period_days = 90 // 数据保留天数
      alert_threshold = 0.85     // 告警阈值
    }
  }
  
  // 路由配置
  route {
    host = "api.example.com"
    path = "/"
    
    upstream {
      service = "api-backend"
      host = "backend.example.com"
      port = 8443
      tls = true
    }
    
    // 路由级别插件配置
    plugins {
      performance_analyzer {
        enabled = true
        sample_rate = 1.0
      }
    }
  }
} 