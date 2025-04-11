// Proksi AI Request Builder Configuration Example

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

    // AI Request Builder plugin configuration
    plugins = [
      {
        name = "ai_request_builder"
        config = {
          // Custom endpoint for the UI (default is /ai-request-builder)
          ui_endpoint = "/api-builder"
          
          // Enable or disable API endpoints for template management
          enable_api = true
          
          // Save request history
          save_history = true
          
          // Maximum history entries to keep
          max_history_entries = 100
          
          // Define provider templates
          templates = [
            // OpenAI template
            {
              name = "OpenAI Chat"
              endpoint = "https://api.openai.com/v1/chat/completions"
              method = "POST"
              headers = {
                "Content-Type" = "application/json"
                "Authorization" = "Bearer ${OPENAI_API_KEY}"
              }
              body_template = <<EOF
{
  "model": "gpt-4",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful AI assistant."
    },
    {
      "role": "user",
      "content": "Hello, how can you help me today?"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 500
}
EOF
              description = "OpenAI GPT-4 chat completions API"
            },
            
            // Anthropic template
            {
              name = "Anthropic Claude"
              endpoint = "https://api.anthropic.com/v1/messages"
              method = "POST"
              headers = {
                "Content-Type" = "application/json"
                "x-api-key" = "${ANTHROPIC_API_KEY}"
                "anthropic-version" = "2023-06-01"
              }
              body_template = <<EOF
{
  "model": "claude-3-opus-20240229",
  "messages": [
    {
      "role": "user",
      "content": "Hello Claude, I'd like your help with a task."
    }
  ],
  "max_tokens": 1000,
  "temperature": 0.7
}
EOF
              description = "Anthropic Claude-3 messages API"
            },
            
            // Google AI template
            {
              name = "Google Gemini"
              endpoint = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent"
              method = "POST"
              headers = {
                "Content-Type" = "application/json"
              }
              body_template = <<EOF
{
  "contents": [
    {
      "parts": [
        {
          "text": "Write a short poem about artificial intelligence."
        }
      ]
    }
  ],
  "generationConfig": {
    "temperature": 0.7,
    "maxOutputTokens": 800,
    "topP": 0.95,
    "topK": 40
  }
}?key=${GOOGLE_API_KEY}
EOF
              description = "Google's Gemini Pro API"
            },
            
            // Azure OpenAI template
            {
              name = "Azure OpenAI"
              endpoint = "https://{your-resource-name}.openai.azure.com/openai/deployments/{deployment-id}/chat/completions?api-version=2023-05-15"
              method = "POST"
              headers = {
                "Content-Type" = "application/json"
                "api-key" = "${AZURE_OPENAI_API_KEY}"
              }
              body_template = <<EOF
{
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful AI assistant."
    },
    {
      "role": "user",
      "content": "What can you tell me about Azure OpenAI?"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 800
}
EOF
              description = "Azure OpenAI Service API"
            }
          ]
        }
      }
    ]
  }
} 