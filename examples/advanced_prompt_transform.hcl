// Advanced Prompt Transformation Example Configuration
// This configuration demonstrates the enhanced prompt transformation capabilities

service_name = "proksi"
worker_threads = 8

// Global configuration
routes = [
  {
    host = "ai-api.example.com"
    
    match_with {
      path = {
        patterns = ["/v1/chat/completions"]
      }
    }
    
    upstreams = [
      {
        ip = "api.openai.com"
        port = 443
      }
    ]
    
    // OpenAI-specific prompt transformations
    plugins = [
      {
        name = "prompt_transform"
        config = {
          config_name = "openai_enhanced"
        }
      }
    ]
  },
  
  {
    host = "ai-api.example.com"
    
    match_with {
      path = {
        patterns = ["/v1/complete"]
      }
    }
    
    upstreams = [
      {
        ip = "api.anthropic.com"
        port = 443
      }
    ]
    
    // Anthropic-specific prompt transformations
    plugins = [
      {
        name = "prompt_transform"
        config = {
          config_name = "anthropic_enhanced"
        }
      }
    ]
  },
  
  {
    host = "ai-api.example.com"
    
    match_with {
      path = {
        patterns = ["/academic/*"]
      }
    }
    
    upstreams = [
      {
        ip = "api.openai.com"
        port = 443
      }
    ]
    
    // Academic context prompt transformations
    plugins = [
      {
        name = "prompt_transform"
        config = {
          config_name = "academic_research"
        }
      }
    ]
  },
  
  {
    host = "ai-api.example.com"
    
    match_with {
      path = {
        patterns = ["/multilingual/*"]
      }
    }
    
    upstreams = [
      {
        ip = "api.openai.com"
        port = 443
      }
    ]
    
    // Multilingual prompt transformations
    plugins = [
      {
        name = "prompt_transform"
        config = {
          config_name = "multilingual"
        }
      }
    ]
  }
]

// 配置高级提示转换
prompt_transform_configs = {
  // OpenAI增强的提示转换配置
  openai_enhanced = {
    transformations = [
      {
        transform_type = "system_message"
        provider = "openai"
        content = "You are a helpful, precise, and concise assistant that provides accurate information. Always prioritize clarity and factual accuracy in your responses."
      },
      {
        transform_type = "add_context"
        provider = "openai"
        content = "The user is looking for concise, well-structured, and accurate information. Focus on providing value and clarity."
      },
      {
        transform_type = "extract_keywords"
        provider = "openai"
        content = "Prioritize responding to these key concepts."
      },
      {
        transform_type = "format_prompt"
        provider = "openai"
        content = "Format the response for clarity"
        format_style = "structured"
      }
    ]
  },
  
  // Anthropic增强的提示转换配置
  anthropic_enhanced = {
    transformations = [
      {
        transform_type = "add_context"
        provider = "anthropic"
        content = "Provide comprehensive answers that are well-structured and easy to follow."
      },
      {
        transform_type = "enhance_content"
        provider = "anthropic"
        content = "Include examples where relevant."
        enhancement_level = "advanced"
      },
      {
        transform_type = "add_safety_check"
        provider = "anthropic"
        content = "Ensure all responses are accurate, balanced, and ethically sound."
      }
    ]
  },
  
  // 学术研究提示转换配置
  academic_research = {
    transformations = [
      {
        transform_type = "system_message"
        content = "You are an academic research assistant with expertise across multiple disciplines. Provide scholarly, nuanced responses with appropriate depth."
      },
      {
        transform_type = "enhance_content"
        content = "Include citations and references where appropriate. Consider alternative viewpoints and methodological limitations."
        enhancement_level = "advanced"
      },
      {
        transform_type = "rag_enhancement"
        content = "Incorporate insights from peer-reviewed literature."
        rag_source = "academic_papers_database"
      },
      {
        transform_type = "format_prompt"
        content = "Format response in an academic style"
        format_style = "structured"
      }
    ]
  },
  
  // 多语言提示转换配置
  multilingual = {
    transformations = [
      {
        transform_type = "translate_prompt"
        content = "Detect and respond in the same language as the query."
        target_lang = "auto_detect"
      },
      {
        transform_type = "custom"
        content = "Add cultural context appropriate to the detected language."
        custom_params = {
          operation = "append"
          add_cultural_context = true
        }
      },
      {
        transform_type = "split_prompt"
        content = "If the response is long, split it into manageable sections."
        max_tokens = 4000
      }
    ]
  }
} 