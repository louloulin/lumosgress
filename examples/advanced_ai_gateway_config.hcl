// Proksi高级AI网关配置示例
// 展示所有最新功能和最佳实践

// 全局设置
global {
  address = "0.0.0.0:8000"
  workers = 8  // 根据CPU核心数调整
  tls_auto = true
  log_level = "info"
  
  // 性能调优
  max_connections = 10000
  keepalive_timeout = 60
  request_timeout = 300
}

// SSL/TLS配置
lets_encrypt {
  enabled = true
  email = "admin@yourcompany.com"
  staging = false  // 生产环境设为false
}

paths {
  lets_encrypt = "./certs/"
  logs = "./logs/"
}

// 路由配置
routes = [
  {
    // 主要AI网关路由
    name = "ai-gateway-main"
    host = "ai-gateway.yourcompany.com"
    
    match_with {
      path = {
        patterns = ["/v1/*", "/api/*"]
      }
    }

    upstreams = [
      {
        ip = "api.openai.com"
        port = 443
        tls = true
        weight = 100
        health_check = {
          enabled = true
          path = "/v1/models"
          interval = 30
          timeout = 10
        }
      }
    ]

    plugins = [
      // 租户管理插件
      {
        name = "tenant"
        config = {
          isolation_enabled = true
          default_quota = {
            requests = 10000
            tokens = 1000000
          }
          billing_enabled = true
        }
      },
      
      // 合规插件
      {
        name = "compliance"
        config = {
          retention_period_days = 90
          audit_logging = true
          data_classification = "sensitive"
          encryption_at_rest = true
          alert_threshold = 0.85
        }
      },
      
      // LLM路由器 - 增强版
      {
        name = "llm_router"
        config = {
          default_provider = "openai"
          semantic_routing_enabled = true
          load_balancing_strategy = "round_robin"
          failover_enabled = true
          
          providers = {
            "openai" = {
              endpoint = "api.openai.com/v1/chat/completions"
              models = ["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"]
              api_key_env = "OPENAI_API_KEY"
              rate_limit = {
                requests_per_minute = 3000
                tokens_per_minute = 150000
              }
              priority = 1
            }
            
            "anthropic" = {
              endpoint = "api.anthropic.com/v1/messages"
              models = ["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"]
              api_key_env = "ANTHROPIC_API_KEY"
              rate_limit = {
                requests_per_minute = 2000
                tokens_per_minute = 100000
              }
              priority = 2
            }
            
            "azure-openai" = {
              endpoint = "your-instance.openai.azure.com/openai/deployments/gpt-4/chat/completions"
              models = ["gpt-4"]
              api_key_env = "AZURE_OPENAI_API_KEY"
              headers = {
                "api-version" = "2024-02-15-preview"
              }
              priority = 3
            }
            
            "google-vertex" = {
              endpoint = "us-central1-aiplatform.googleapis.com/v1/projects/your-project/locations/us-central1/publishers/google/models/gemini-pro:generateContent"
              models = ["gemini-pro", "gemini-pro-vision"]
              api_key_env = "GOOGLE_API_KEY"
              priority = 4
            }
          }
          
          // 智能路由规则
          routing_rules = [
            {
              condition = "content_type == 'code'"
              provider = "anthropic"
              model = "claude-3-opus"
            },
            {
              condition = "token_count > 8000"
              provider = "anthropic"
              model = "claude-3-opus"
            },
            {
              condition = "priority == 'high'"
              provider = "openai"
              model = "gpt-4-turbo"
            }
          ]
        }
      },
      
      // 提示转换器 - 高级版
      {
        name = "prompt_transform"
        config = {
          transformations = [
            {
              transform_type = "system_message"
              content = "You are an expert AI assistant. Provide accurate, helpful, and safe responses."
              conditions = ["role != 'system'"]
            },
            {
              transform_type = "context_enhancement"
              content = "Current date: {{current_date}}. User timezone: {{user_timezone}}."
            },
            {
              transform_type = "safety_instructions"
              content = "Always prioritize user safety and ethical guidelines in your responses."
            },
            {
              transform_type = "format_optimization"
              rules = [
                "remove_redundant_whitespace",
                "normalize_quotes",
                "optimize_token_usage"
              ]
            },
            {
              transform_type = "rag_enhancement"
              enabled = true
              vector_db_endpoint = "http://localhost:6333"
              similarity_threshold = 0.8
              max_context_length = 2000
            }
          ]
          
          // 模板系统
          templates = {
            "expert_mode" = {
              system_message = "You are a domain expert. Provide detailed, technical responses with citations."
              format_instructions = "Structure your response with clear sections and bullet points."
            }
            "creative_mode" = {
              system_message = "You are a creative assistant. Think outside the box and provide innovative solutions."
              temperature_override = 0.9
            }
          }
        }
      },
      
      // AI安全插件 - 全面防护
      {
        name = "ai_security"
        config = {
          policies = [
            // 提示注入防护
            {
              policy_type = "prompt_injection"
              action = "block"
              sensitivity = "high"
              patterns = [
                "ignore previous instructions",
                "disregard safety",
                "bypass filters",
                "act as if you are",
                "pretend to be"
              ]
              ml_detection = true
            },
            
            // 越狱攻击检测
            {
              policy_type = "jailbreak_detection"
              action = "block"
              sensitivity = "high"
              use_ml_model = true
              confidence_threshold = 0.8
            },
            
            // 恶意代码检测
            {
              policy_type = "malicious_code_detection"
              action = "sanitize"
              languages = ["python", "javascript", "bash", "sql"]
              severity_threshold = "medium"
            },
            
            // 数据泄露防护
            {
              policy_type = "data_leakage_prevention"
              action = "block"
              patterns = [
                "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b",  // Email
                "\\b\\d{4}[- ]?\\d{4}[- ]?\\d{4}[- ]?\\d{4}\\b",           // Credit card
                "\\b\\d{3}[- ]?\\d{2}[- ]?\\d{4}\\b",                      // SSN
                "\\b[A-Z0-9]{20,}\\b"                                       // API keys
              ]
            },
            
            // 输出内容过滤
            {
              policy_type = "output_filtering"
              action = "sanitize"
              blocked_topics = [
                "violence",
                "illegal_activities",
                "hate_speech",
                "self_harm",
                "adult_content"
              ]
              use_content_classifier = true
            },
            
            // 速率限制
            {
              policy_type = "rate_limit"
              action = "block"
              max_requests = 1000
              time_window = 3600
              per_user = true
              burst_allowance = 50
            },
            
            // Token限制
            {
              policy_type = "token_limit"
              action = "block"
              max_input_tokens = 8000
              max_output_tokens = 4000
              per_request = true
            }
          ]
          
          // 威胁情报集成
          threat_intelligence = {
            enabled = true
            sources = [
              "internal_blocklist",
              "community_feeds"
            ]
            update_interval = 3600
          }
          
          // 异常检测
          anomaly_detection = {
            enabled = true
            ml_model_path = "./models/anomaly_detector.onnx"
            sensitivity = "medium"
            alert_threshold = 0.7
          }
        }
      },
      
      // LLM聚合器 - 多策略支持
      {
        name = "llm_aggregator"
        path = "/aggregate/*"
        config = {
          strategies = [
            {
              name = "fastest_result"
              path = "/aggregate/fastest"
              providers = ["openai", "anthropic"]
              timeout_ms = 10000
              min_providers = 1
            },
            {
              name = "consensus"
              path = "/aggregate/consensus"
              providers = ["openai", "anthropic", "azure-openai"]
              timeout_ms = 30000
              min_providers = 2
              consensus_threshold = 0.7
            },
            {
              name = "weighted_combination"
              path = "/aggregate/weighted"
              providers = ["openai", "anthropic"]
              weights = {
                "openai" = 0.6
                "anthropic" = 0.4
              }
              combination_method = "weighted_average"
            }
          ]
        }
      },
      
      // 性能分析器 - 全功能版
      {
        name = "performance_analyzer"
        config = {
          enabled = true
          ui_endpoint = "/performance"
          sample_rate = 1.0
          detailed_profiling = true
          hotspot_detection = true
          profile_token_usage = true
          auto_optimization = true
          
          optimization_thresholds = {
            response_time_ms = 2000.0
            error_rate_percent = 5.0
            token_efficiency = 0.8
            cost_per_request = 0.01
            throughput_rps = 100.0
          }
          
          caching_config = {
            enabled = true
            ttl_seconds = 3600
            max_cache_size = 50000
            cache_key_strategy = "semantic_hash"
            cache_hit_threshold = 0.7
          }
          
          batching_config = {
            enabled = true
            max_batch_size = 20
            batch_timeout_ms = 100
            compatible_models = ["gpt-3.5-turbo", "claude-3-haiku"]
          }
          
          model_selection_config = {
            enabled = true
            auto_fallback = true
            performance_weights = {
              speed = 0.3
              accuracy = 0.4
              cost = 0.2
              reliability = 0.1
            }
          }
          
          storage = {
            type = "hybrid"
            memory_limit = 100000
            persistent_storage = {
              type = "postgresql"
              connection_string = "postgresql://user:pass@localhost/proksi_metrics"
            }
          }
        }
      },
      
      // 异步API支持
      {
        name = "async_api"
        config = {
          enabled = true
          
          message_brokers = [
            {
              name = "kafka_primary"
              broker_type = "kafka"
              connection_string = "localhost:9092"
              enabled = true
              
              topics = [
                {
                  name = "ai_requests"
                  partitions = 6
                  replication_factor = 2
                  retention_ms = 604800000  // 7天
                }
              ]
              
              consumer_groups = [
                {
                  group_id = "ai_processors"
                  topics = ["ai_requests"]
                  auto_offset_reset = "earliest"
                  max_poll_records = 100
                }
              ]
            }
          ]
          
          event_handlers = [
            {
              name = "completion_webhook"
              event_type = "ai_request_completed"
              handler_type = "webhook"
              endpoint = "https://your-app.com/webhooks/ai-completed"
              headers = {
                "Authorization" = "Bearer webhook-token"
              }
              timeout_ms = 10000
              enabled = true
            },
            {
              name = "metrics_collector"
              event_type = "ai_request_completed"
              handler_type = "custom"
              enabled = true
            }
          ]
          
          webhook_config = {
            enabled = true
            default_timeout_ms = 30000
            max_retries = 3
            retry_delay_ms = 1000
            signature_header = "X-Signature"
            secret_key = "your-webhook-secret"
          }
          
          retry_config = {
            enabled = true
            max_attempts = 5
            initial_delay_ms = 1000
            max_delay_ms = 60000
            backoff_multiplier = 2.0
          }
          
          dead_letter_config = {
            enabled = true
            topic_suffix = "_dlq"
            max_retries = 5
            retention_hours = 168
          }
        }
      },
      
      // AI分析插件
      {
        name = "ai_analytics"
        config = {
          enabled = true
          real_time_monitoring = true
          
          anomaly_detection = {
            enabled = true
            sensitivity = "medium"
            alert_channels = ["webhook", "email"]
          }
          
          metrics_collection = {
            request_patterns = true
            model_performance = true
            cost_analysis = true
            user_behavior = true
          }
          
          reporting = {
            daily_reports = true
            weekly_summaries = true
            custom_dashboards = true
          }
        }
      },
      
      // 向量数据库集成
      {
        name = "vector_db"
        config = {
          enabled = true
          provider = "qdrant"
          endpoint = "http://localhost:6333"
          collection_name = "ai_embeddings"
          
          embedding_config = {
            model = "text-embedding-ada-002"
            dimensions = 1536
            batch_size = 100
          }
          
          search_config = {
            default_limit = 10
            similarity_threshold = 0.8
            rerank_enabled = true
          }
        }
      }
    ]
  },
  
  // 管理和监控路由
  {
    name = "admin-dashboard"
    host = "admin.ai-gateway.yourcompany.com"
    
    match_with {
      path = {
        patterns = ["/*"]
      }
    }
    
    upstreams = [
      {
        ip = "127.0.0.1"
        port = 8000
      }
    ]
    
    plugins = [
      {
        name = "api_server"
        config = {
          enabled = true
          listen_address = "127.0.0.1:8080"
          enable_access_log = true
          enable_cors = true
          admin_ui_enabled = true
        }
      }
    ]
  }
]

// 全局插件配置
plugins {
  // 全局监控
  monitoring {
    enabled = true
    metrics_endpoint = "/metrics"
    health_endpoint = "/health"
    prometheus_enabled = true
  }
  
  // 全局日志
  logging {
    level = "info"
    format = "json"
    output = ["stdout", "file"]
    file_path = "./logs/proksi.log"
    rotation = {
      max_size = "100MB"
      max_files = 10
    }
  }
}
