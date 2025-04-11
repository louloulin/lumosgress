// Vector database operations example
const { ProksiClient } = require('proksi-sdk');

// Initialize the client
const client = new ProksiClient({
  baseUrl: process.env.PROKSI_API_URL || 'http://localhost:8000',
  apiKey: process.env.PROKSI_API_KEY,
});

// Sample vector data
const sampleVectors = [
  {
    id: 'doc-1',
    values: Array.from({ length: 4 }, () => Math.random()),
    metadata: { title: 'Introduction to AI', category: 'education' }
  },
  {
    id: 'doc-2',
    values: Array.from({ length: 4 }, () => Math.random()),
    metadata: { title: 'Machine Learning Basics', category: 'education' }
  },
  {
    id: 'doc-3',
    values: Array.from({ length: 4 }, () => Math.random()),
    metadata: { title: 'Advanced Deep Learning', category: 'research' }
  }
];

// Sample query vector
const queryVector = Array.from({ length: 4 }, () => Math.random());

async function main() {
  try {
    // Step 1: Upsert vectors
    console.log('Upserting vectors...');
    await client.upsertVectors({
      namespace: 'documents',
      vectors: sampleVectors
    });
    console.log('Vectors uploaded successfully');
    
    // Step 2: Search vectors
    console.log('\nSearching for similar vectors...');
    const searchResults = await client.searchVectors({
      namespace: 'documents',
      query_vector: queryVector,
      top_k: 2
    });
    
    console.log('Search results:');
    searchResults.results.forEach((result, i) => {
      console.log(`${i+1}. ID: ${result.id}, Score: ${result.score}`);
      console.log(`   Metadata: ${JSON.stringify(result.metadata)}`);
    });
    
    // Step 3: Delete a vector
    console.log('\nDeleting a vector...');
    await client.deleteVectors({
      namespace: 'documents',
      ids: ['doc-1']
    });
    console.log('Vector deleted successfully');
    
    // Step 4: Verify deletion by searching again
    console.log('\nVerifying deletion with another search...');
    const verificationResults = await client.searchVectors({
      namespace: 'documents',
      query_vector: queryVector,
      top_k: 3
    });
    
    console.log('New search results:');
    verificationResults.results.forEach((result, i) => {
      console.log(`${i+1}. ID: ${result.id}, Score: ${result.score}`);
    });
    
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main(); 