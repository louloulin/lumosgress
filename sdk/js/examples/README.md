# Proksi SDK Examples

This directory contains examples demonstrating how to use the Proksi JavaScript SDK.

## Running the Examples

Before running the examples, set up the environment variables:

```bash
export PROKSI_API_URL="https://your-proksi-instance.com"
export PROKSI_API_KEY="your-api-key"
```

Or create a `.env` file in this directory with these variables.

### Basic Completion

A simple example showing how to send a completion request:

```bash
node basic-completion.js
```

### Streaming Completion

Example of using streaming for real-time responses:

```bash
node streaming-completion.js
```

### Vector Operations

Demonstrates vector database operations (upsert, search, delete):

```bash
node vector-operations.js
```

## Using with Local Proksi Instance

If you're running a Proksi instance locally, set:

```bash
export PROKSI_API_URL="http://localhost:8000"
```

## Troubleshooting

If you encounter errors:

1. Verify that your Proksi instance is running and accessible
2. Check that your API key is correct
3. Ensure that the requested model is available in your Proksi configuration
4. Check the Proksi logs for more detailed error information 