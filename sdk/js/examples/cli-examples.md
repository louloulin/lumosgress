# Proksi CLI Examples

The Proksi SDK includes a powerful command-line interface (CLI) that allows you to interact with the Proksi AI Gateway directly from your terminal.

## Installation

You can install the CLI globally:

```bash
npm install -g proksi-sdk
```

Or use it with npx:

```bash
npx proksi-sdk
```

## Configuration

The CLI uses environment variables for configuration:

```bash
# Set these in your .env file or export them in your shell
export PROKSI_API_URL="https://your-proksi-instance.com"
export PROKSI_API_KEY="your-api-key"
```

## Basic Usage

### Getting Help

```bash
# Show general help
proksi --help

# Show help for a specific command
proksi completion --help
```

### Sending Completion Requests

```bash
# Basic completion request
proksi completion "What is the capital of France?"

# Specify a provider and model
proksi completion --provider openai --model gpt-4 "Explain quantum computing in simple terms."

# Add a system message
proksi completion --system "You are a helpful AI assistant specialized in geology." "What causes earthquakes?"

# Save the response to a file
proksi completion --output response.txt "Write a short story about a robot learning to feel emotions."
```

### Streaming Responses

```bash
# Stream a completion
proksi stream "Write a poem about artificial intelligence."

# Stream with specific parameters
proksi stream --provider anthropic --model claude-3-opus --temperature 0.9 "Tell me a joke about programming."
```

### Interactive Chat Mode

```bash
# Start an interactive chat session
proksi chat

# Configure the chat session
proksi chat --provider openai --model gpt-4 --system "You are a helpful AI assistant specialized in Python programming."
```

### Vector Database Operations

```bash
# Upsert a vector
proksi vector:upsert --namespace products --id product-123 0.1 0.2 0.3 0.4

# Upsert with metadata
proksi vector:upsert --namespace products --id product-123 --metadata '{"name":"Product 123","category":"electronics"}' 0.1 0.2 0.3 0.4

# Search vectors
proksi vector:search --namespace products 0.1 0.2 0.3 0.4

# Search with top-k and filter
proksi vector:search --namespace products --top-k 10 --filter '{"category":"electronics"}' 0.1 0.2 0.3 0.4

# Delete vectors
proksi vector:delete --namespace products product-123 product-456
```

## Advanced Usage

### Loading Vectors from Files

You can store your vectors in JSON files for easier management:

```bash
# Create a vector file
echo '[0.1, 0.2, 0.3, 0.4]' > vector.json

# Upsert using the file
proksi vector:upsert --namespace products --id product-123 --file vector.json

# Search using the file
proksi vector:search --namespace products --file vector.json
```

### Scripting with the CLI

You can use the CLI in shell scripts:

```bash
#!/bin/bash
# Example: Process a list of queries

QUERIES=("What is machine learning?" "Explain neural networks." "What is deep learning?")

for query in "${QUERIES[@]}"; do
  echo "Processing query: $query"
  proksi completion --output "responses/$(echo $query | tr ' ' '_').txt" "$query"
  sleep 1
done
```

## Tips and Tricks

1. Use `--help` with any command to see all available options.
2. For chat mode, type `exit` or press `Ctrl+C` to end the session.
3. Use the `--output` option to save responses to files for later analysis.
4. When working with vectors, use the `--file` option for complex vectors with many dimensions.
5. Set environment variables in your `.env` file for easier configuration. 