// Basic completion example
const { ProksiClient } = require('proksi-sdk');

// Initialize the client
const client = new ProksiClient({
  baseUrl: process.env.PROKSI_API_URL || 'http://localhost:8000',
  apiKey: process.env.PROKSI_API_KEY,
  defaultProvider: 'openai',
  defaultModel: 'gpt-3.5-turbo'
});

async function main() {
  try {
    console.log('Sending completion request...');
    
    const response = await client.completion({
      messages: [
        { role: 'system', content: 'You are a helpful assistant.' },
        { role: 'user', content: 'Explain what an AI gateway is in one sentence.' }
      ],
      temperature: 0.7,
      max_tokens: 100
    });
    
    console.log('\nResponse:');
    console.log(`Model: ${response.model}`);
    console.log(`Provider: ${response.provider}`);
    console.log(`Content: ${response.message.content}`);
    console.log(`Tokens: ${response.usage.total_tokens}`);
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main(); 