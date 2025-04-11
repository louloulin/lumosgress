# Prompt Debugger Plugin

The Prompt Debugger plugin provides tools for analyzing and improving AI prompts. It helps developers write more effective prompts by detecting common issues and suggesting improvements.

## Features

- **Pattern-Based Analysis**: Identify common prompt issues using configurable rules
- **Real-time Feedback**: Get immediate suggestions for prompt improvements
- **Interactive UI**: User-friendly interface for prompt testing and analysis
- **History Tracking**: Save analysis history for reference
- **Rule Management**: View and customize analysis rules
- **API Support**: Analyze prompts programmatically

## Configuration

Add the plugin to your Proksi route configuration:

```hcl
plugins = [
  {
    name = "prompt_debugger"
    config = {
      // Optional: Custom endpoint for the UI (default is /prompt-debugger)
      ui_endpoint = "/prompt-debug"
      
      // Optional: Enable or disable API endpoints (default is true)
      enable_api = true
      
      // Optional: Maximum history entries to keep (default is 100)
      max_history_entries = 100
      
      // Optional: Intercept and analyze LLM requests in real-time (default is false)
      intercept_requests = false
      
      // Optional: Endpoint for fetching custom rules (default is none)
      custom_rules_endpoint = "https://example.com/rules"
      
      // Optional: Custom analysis rules
      rules = [
        {
          name = "System Role First"
          description = "Check if system message appears before user messages"
          pattern = "\"role\":\\s*\"system\""
          suggestion = "Place system messages at the beginning of your conversation to establish context."
          severity = "warning"
        },
        // Additional rules...
      ]
    }
  }
]
```

## Default Rules

The plugin comes with the following default rules:

1. **System Role First**: Check if system message appears before user messages
2. **Clear Instructions**: Check if instructions are clear and specific
3. **Long Prompt**: Detect excessively long prompts
4. **Vague Language**: Identify vague or ambiguous language
5. **Few-Shot Example**: Check if few-shot examples are provided

## Usage

1. **Access the UI**: Navigate to the configured endpoint (default: `/prompt-debugger`) in your browser
2. **Enter a Prompt**: Type or paste your AI prompt
3. **Analyze**: Get immediate feedback on potential issues
4. **Review Suggestions**: See improvement recommendations
5. **View History**: Access past analyses for reference

## API Reference

When `enable_api` is set to `true`, the following API endpoints are available:

- `POST /prompt-debugger/api/analyze`: Analyze a prompt
  - Request body: Raw prompt text or JSON with LLM provider format
  - Response: Analysis results with matches and suggestions
- `GET /prompt-debugger/api/rules`: Get the configured debug rules
- `GET /prompt-debugger/api/history`: Get analysis history

Replace `/prompt-debugger` with your custom endpoint if configured.

## Request Interception

When `intercept_requests` is set to `true`, the plugin can analyze AI prompts in transit to LLM providers. This feature:

- Detects potential issues before the request reaches the LLM
- Logs analysis results for monitoring
- Allows for prompt improvement suggestions in real-time

## Example Configuration

```hcl
plugins = [
  {
    name = "prompt_debugger"
    config = {
      ui_endpoint = "/prompt-tools"
      rules = [
        {
          name = "Output Format"
          description = "Check if output format is specified"
          pattern = "(JSON|XML|markdown|table|list|format)"
          suggestion = "Consider specifying the desired output format for more structured responses."
          severity = "info"
        },
        {
          name = "Context Length"
          description = "Check for sufficient context"
          pattern = ".{500,}"
          suggestion = "Your prompt has good context length, which helps the model understand the task."
          severity = "info"
        }
      ]
    }
  }
]
``` 