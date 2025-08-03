
![GitHub Release](https://img.shields.io/github/v/release/luizfonseca/proksi?style=for-the-badge)
![Crates.io MSRV](https://img.shields.io/crates/msrv/proksi?style=for-the-badge)
![Crates.io License](https://img.shields.io/crates/l/proksi?style=for-the-badge)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/proksi?style=for-the-badge)](https://crates.io/crates/proksi)

# Proksi: Automatic SSL, HTTP, and DNS Proxy

<img src="./assets/discord.png" alt="discord-logo" width="200"/>


# About

Proksi is a simple, lightweight, and easy-to-use proxy server that automatically handles SSL, HTTP, and DNS traffic. It is designed to be used as a standalone proxy server or as a component in a larger system. Proksi is written in [Rust](https://www.rust-lang.org/) and uses [Pingora](https://github.com/cloudflare/pingora) as its core networking library.


# Features

Proksi is a next-generation AI Gateway and reverse proxy that offers comprehensive features for managing AI/LLM traffic and traditional web services:

## 🤖 AI Gateway Features
- **Multi-LLM Provider Support**: OpenAI, Anthropic, Google Vertex AI, Azure OpenAI, and more
- **Intelligent Routing**: Semantic-based routing to optimal models based on content analysis
- **Advanced Prompt Engineering**: Automatic prompt enhancement, transformation, and optimization
- **AI Security Suite**: Multi-layered protection against prompt injection, jailbreaks, and malicious content
- **Model Aggregation**: Combine responses from multiple providers or select fastest/best results
- **Performance Optimization**: Smart caching, request batching, and automatic performance tuning
- **Async API Support**: Message queue integration (Kafka, RabbitMQ, Redis) for high-throughput scenarios

## 🔒 Security & Compliance
- **Advanced Threat Detection**: ML-powered detection of prompt injection and jailbreak attempts
- **Data Loss Prevention**: Automatic detection and blocking of sensitive information
- **Content Filtering**: Output sanitization and topic-based content blocking
- **Audit Logging**: Comprehensive logging for compliance and security analysis
- **Multi-tenant Isolation**: Secure tenant separation with quota management

## ⚡ Performance & Scalability
- **Intelligent Caching**: Semantic hashing for optimal cache hit rates
- **Load Balancing**: Advanced load balancing with health checks and failover
- **Real-time Analytics**: Performance monitoring with automatic optimization suggestions
- **Horizontal Scaling**: Support for distributed deployments and clustering

## 🛠 Traditional Proxy Features
- Automatic Docker and Docker Swarm service discovery through labels
- Built-in middlewares: OAuth, JWT, Rate Limiting, CDN Caching, and more
- Single binary deployment with minimal resource requirements
- Automatic SSL through Let's Encrypt with HTTP to HTTPS redirection
- Configuration through **HCL** with support for functions and environment variables
- Powerful plugin system using **WebAssembly (WASM)** for custom extensions

# Quick Start

## AI Gateway Quick Start

Get started with Proksi AI Gateway in under 5 minutes:

### 1. Install Proksi
```bash
# Linux/macOS
curl -fsSL https://github.com/luizfonseca/proksi/releases/latest/download/install.sh | sh

# Or download manually from releases
# https://github.com/luizfonseca/proksi/releases
```

### 2. Set up your API keys
```bash
export OPENAI_API_KEY="your-openai-api-key"
export ANTHROPIC_API_KEY="your-anthropic-api-key"  # optional
```

### 3. Create a basic AI Gateway configuration
Create a file named `ai-gateway.hcl`:

```hcl
global {
  address = "0.0.0.0:8000"
  workers = 4
}

routes = [
  {
    host = "ai-gateway.localhost"

    match_with {
      path = { patterns = ["/v1/*"] }
    }

    upstreams = [{ ip = "api.openai.com", port = 443, tls = true }]

    plugins = [
      {
        name = "llm_router"
        config = {
          default_provider = "openai"
          providers = {
            "openai" = {
              endpoint = "api.openai.com/v1/chat/completions"
              models = ["gpt-3.5-turbo", "gpt-4"]
              api_key_env = "OPENAI_API_KEY"
            }
          }
        }
      },
      {
        name = "ai_security"
        config = {
          policies = [
            {
              policy_type = "prompt_injection"
              action = "block"
            }
          ]
        }
      }
    ]
  }
]
```

### 4. Start the gateway
```bash
proksi -c ai-gateway.hcl
```

### 5. Test your AI Gateway
```bash
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Host: ai-gateway.localhost" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Traditional Proxy Quick Start

For traditional reverse proxy usage:

```hcl
lets_encrypt {
  enabled = true
  email = "my@email.com"
}

routes = [
  {
    host = "mysite.localhost"
    upstreams = [{
      ip = "docs.proksi.info"
      port = 443
    }]
  }
]
```

Run with: `proksi -c proksi.hcl`

# Documentation

## 📚 Comprehensive Guides
- **[Quick Start Guide](./docs/quick_start_guide.md)** - Get up and running in 5 minutes
- **[AI Gateway Usage](./docs/ai_gateway_usage.md)** - Complete AI Gateway configuration guide
- **[Performance Optimization](./docs/performance_optimization.md)** - Advanced performance tuning
- **[Async API Guide](./docs/async_api_guide.md)** - Asynchronous processing with message queues
- **[Security Configuration](./docs/security_guide.md)** - Advanced security features

## 🔧 Configuration Examples
- **[Basic AI Gateway](./examples/ai_gateway_config.hcl)** - Simple AI Gateway setup
- **[Advanced Configuration](./examples/advanced_ai_gateway_config.hcl)** - Full-featured setup
- **[Plugin Examples](./examples/plugins_config.hcl)** - Plugin system examples

## 🌐 Online Documentation
Full documentation is available at [https://docs.proksi.info](https://docs.proksi.info)


# Contributing
We welcome contributions to Proksi. If you have any **suggestions** or **ideas**, please feel free to open an issue or a pull request on the GitHub repository.

# License
Proksi is licensed under the [MIT License](https://github.com/luizfonseca/proksi/blob/main/LICENSE), the [Apache License 2.0](https://github.com/luizfonseca/proksi/blob/main/LICENSE-APACHE) and is free to use and modify.
