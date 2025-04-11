# AI Request Builder Plugin

The AI Request Builder plugin provides a user-friendly web interface for constructing and testing API requests to various LLM providers. It helps developers quickly prototype and test their AI requests without needing external tools.

## Features

- **Visual Request Builder**: Interactive UI for building and sending AI API requests
- **Provider Templates**: Pre-configured templates for popular LLM providers (OpenAI, Anthropic, Google, Azure)
- **Request History**: Save and reuse successful requests
- **Custom Headers and Body**: Full control over request parameters
- **Real-time Testing**: Send requests and view responses directly in the UI

## Configuration

Add the plugin to your Proksi route configuration:

```hcl
plugins = [
  {
    name = "ai_request_builder"
    config = {
      // Optional: Custom endpoint for the UI (default is /ai-request-builder)
      ui_endpoint = "/api-builder"
      
      // Optional: Enable or disable API endpoints (default is true)
      enable_api = true
      
      // Optional: Save request history (default is true)
      save_history = true
      
      // Optional: Maximum history entries to keep (default is 100)
      max_history_entries = 100
      
      // Optional: Define custom provider templates
      templates = [
        {
          name = "OpenAI Chat"
          endpoint = "https://api.openai.com/v1/chat/completions"
          method = "POST"
          headers = {
            "Content-Type" = "application/json"
            "Authorization" = "Bearer ${OPENAI_API_KEY}"
          }
          body_template = "{ ... }"
          description = "OpenAI GPT-4 chat completions API"
        }
        // Additional templates...
      ]
    }
  }
]
```

## Usage

1. **Access the UI**: Navigate to the configured endpoint (default: `/ai-request-builder`) in your browser
2. **Select a Template**: Choose from pre-configured LLM provider templates
3. **Customize Request**: Modify headers, endpoint, and request body as needed
4. **Send Request**: Test your request and view the response
5. **Save for Reuse**: Request history is automatically saved for future reference

## Provider Templates

The plugin comes with pre-configured templates for:

1. **OpenAI**: Chat completions API
2. **Anthropic Claude**: Messages API
3. **Google Gemini**: GenerateContent API
4. **Azure OpenAI**: Chat completions API

You can add custom templates via the configuration.

## API Reference

When `enable_api` is set to `true`, the following API endpoints are available:

- `GET /ai-request-builder/api/templates`: List all available templates
- `GET /ai-request-builder/api/history`: Get request history
- `POST /ai-request-builder/api/execute`: Execute a request and return the result

Replace `/ai-request-builder` with your custom endpoint if configured.

## Example Configuration

See the full example configuration in `examples/ai_request_builder_config.hcl`.

## Security Considerations

- API keys are stored in the browser and not on the server
- Environment variables in templates (e.g., `${OPENAI_API_KEY}`) need to be replaced with actual values by the client
- Consider restricting access to the Request Builder UI in production environments 