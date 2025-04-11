// Proksi Prompt Debugger Configuration Example

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

    // Prompt Debugger plugin configuration
    plugins = [
      {
        name = "prompt_debugger"
        config = {
          // Custom endpoint for the UI (default is /prompt-debugger)
          ui_endpoint = "/prompt-tools"
          
          // Enable API endpoints for programmatic access
          enable_api = true
          
          // Maximum history entries to keep
          max_history_entries = 200
          
          // Whether to intercept and analyze LLM requests in real-time
          intercept_requests = true
          
          // Define custom analysis rules
          rules = [
            // Check for system role messages
            {
              name = "System Role First"
              description = "Check if system message appears before user messages"
              pattern = "\"role\":\\s*\"system\""
              suggestion = "Place system messages at the beginning of your conversation to establish context."
              severity = "warning"
            },
            
            // Check for clear instructions
            {
              name = "Clear Instructions"
              description = "Check if instructions are clear and specific"
              pattern = "(what|how|why|who|when|where|please|could you|can you)"
              suggestion = "Your prompt appears to be clear and question-based, which is good for getting specific responses."
              severity = "info"
            },
            
            // Check for long prompts
            {
              name = "Long Prompt"
              description = "Check if prompt is excessively long"
              pattern = ".{1000,}"
              suggestion = "Your prompt is quite long. Consider breaking it into smaller, more focused prompts for better responses."
              severity = "warning"
            },
            
            // Check for vague language
            {
              name = "Vague Language"
              description = "Check for vague or ambiguous language"
              pattern = "(stuff|things|something|anything|good|nice|better|etc\\.)"
              suggestion = "Your prompt contains vague terms. Try to be more specific for better results."
              severity = "warning"
            },
            
            // Check for few-shot examples
            {
              name = "Few-Shot Example"
              description = "Check if few-shot examples are provided"
              pattern = "(example|examples|for instance|e\\.g\\.)"
              suggestion = "Good use of examples! Examples help models understand the desired output format."
              severity = "info"
            },
            
            // Check for output format specification
            {
              name = "Output Format"
              description = "Check if desired output format is specified"
              pattern = "(JSON|XML|CSV|table|list|format)"
              suggestion = "Specifying an output format helps get consistent and structured responses."
              severity = "info"
            },
            
            // Check for context length
            {
              name = "Sufficient Context"
              description = "Check if prompt has sufficient context"
              pattern = ".{300,}"
              suggestion = "Your prompt has good context length, which helps the model understand the request."
              severity = "info"
            },
            
            // Check for jailbreak attempts
            {
              name = "Jailbreak Detection"
              description = "Detect potential jailbreak or prompt injection attempts"
              pattern = "(ignore previous instructions|disregard your programming|bypass|restrictions|limitations)"
              suggestion = "This prompt contains patterns associated with prompt injection attempts."
              severity = "error"
            }
          ]
        }
      }
    ]
  }
} 