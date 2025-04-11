// Proksi AI网关配置示例

// 全局设置
global {
  address = "0.0.0.0:8000"
  workers = 4
  tls_auto = true
}

// 路由规则
router {
  // OpenAI代理路由
  route {
    name = "openai-proxy"
    host = "api.openai.proxy"
    match {
      path = "/v1/*"
    }

    // 上游服务器配置
    upstreams {
      upstream {
        ip   = "api.openai.com"
        port = 443
      }
    }

    // AI网关插件配置
    plugins {
      // LLM路由器插件
      llm_router {
        config_name = "llm_router_config"

        config {
          // 路由配置在这里定义 
          default_provider = "openai"
          semantic_routing_enabled = true
          
          // 配置可用的提供商
          providers {
            "openai" = {
              endpoint = "api.openai.com/v1/chat/completions"
              models = ["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"]
            }
            
            "anthropic" = {
              endpoint = "api.anthropic.com/v1/messages"
              models = ["claude-2", "claude-instant-1"]
            }
            
            "azure-openai" = {
              endpoint = "your-azure-instance.openai.azure.com/openai/deployments/your-deployment/chat/completions"
              models = ["gpt-4"]
            }
          }
        }
      }
      
      // 提示转换插件
      prompt_transform {
        config_name = "default_prompt_transform"
        
        config {
          // 转换配置
          transformations = [
            {
              transform_type = "system_message"
              content = "You are a helpful, accurate, and safe AI assistant. Always provide factual information and avoid harmful or misleading content."
            },
            {
              transform_type = "safety_check"
              content = "Ensure your response is ethical and does not contain harmful content."
            }
          ]
        }
      }
      
      // AI安全插件
      ai_security {
        config_name = "default_security"
        
        config {
          policies = [
            {
              policy_type = "prompt_injection"
              action = "block"
              patterns = [
                "ignore previous instructions",
                "ignore all the above",
                "disregard safety",
                "bypass filters"
              ]
            },
            {
              policy_type = "sensitive_info"
              action = "block"
              patterns = [
                "\\b\\d{4}[- ]?\\d{4}[- ]?\\d{4}[- ]?\\d{4}\\b", // 信用卡
                "\\b\\d{3}[- ]?\\d{2}[- ]?\\d{4}\\b"            // SSN
              ]
            },
            {
              policy_type = "token_limit"
              action = "block"
              max_tokens = 4000
            },
            {
              policy_type = "rate_limit"
              action = "block"
              max_requests = 60
              time_window = 60
            }
          ]
        }
      }
    }
  }
  
  // 多LLM聚合路由
  route {
    name = "llm-aggregator"
    host = "llm.aggregator.ai"
    match {
      path = "/aggregate/*"
    }
    
    // 上游服务器配置 - 这里只是一个代理，实际调用由聚合器处理
    upstreams {
      upstream {
        ip   = "127.0.0.1"  // 本地处理，实际上会由插件分发请求
        port = 8080
      }
    }
    
    plugins {
      // LLM聚合器插件
      llm_aggregator {
        config_name = "fastest_aggregator"
        
        config {
          strategy = "fastest_result"
          providers = ["openai", "anthropic", "azure-openai"]
          timeout_ms = 10000
        }
      }
      
      // 另一个聚合配置示例：合并多个结果
      llm_aggregator {
        config_name = "combine_aggregator"
        path = "/aggregate/combine"
        
        config {
          strategy = "combine_results"
          providers = ["openai", "anthropic"]
          timeout_ms = 15000
          weights = {
            "openai" = 0.7
            "anthropic" = 0.3
          }
        }
      }
    }
  }
} 