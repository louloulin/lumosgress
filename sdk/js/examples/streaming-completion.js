// Streaming completion example
const { ProksiClient } = require('proksi-sdk');

// Initialize the client
const client = new ProksiClient({
  baseUrl: process.env.PROKSI_API_URL || 'http://localhost:8000',
  apiKey: process.env.PROKSI_API_KEY,
});

console.log('Starting streaming completion...\n');

// Start the streaming completion
client.streamCompletion(
  {
    provider: 'anthropic',
    model: 'claude-3-haiku',
    messages: [
      { role: 'user', content: 'Generate a short poem about artificial intelligence' }
    ],
    temperature: 0.7,
    max_tokens: 150,
  },
  (chunk) => {
    // Process each chunk as it arrives
    if (chunk.delta.content) {
      process.stdout.write(chunk.delta.content);
    }
  },
  (error) => {
    console.error('\nError:', error.message);
  },
  () => {
    console.log('\n\nStream completed');
  }
); 