import axios from 'axios';
import { ProksiClient } from '../client';
import { VectorUpsertRequest, VectorSearchRequest, VectorSearchResponse } from '../types';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('ProksiClient Vector Operations', () => {
  let client: ProksiClient;

  beforeEach(() => {
    jest.clearAllMocks();
    
    // Setup axios create mock
    mockedAxios.create.mockReturnValue(mockedAxios as any);
    
    // Initialize client
    client = new ProksiClient({ 
      baseUrl: 'https://api.example.com',
      apiKey: 'test-api-key'
    });
  });

  describe('upsertVectors', () => {
    test('should successfully upsert vectors', async () => {
      const mockRequest: VectorUpsertRequest = {
        collection: 'test-collection',
        provider: 'pinecone',
        vectors: [
          {
            id: 'vec-1',
            values: [0.1, 0.2, 0.3],
            metadata: { source: 'test' }
          },
          {
            id: 'vec-2',
            values: [0.4, 0.5, 0.6],
            metadata: { source: 'test' }
          }
        ]
      };
      
      const mockResponse = {
        message: 'Vectors inserted successfully',
        count: 2
      };
      
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });
      
      const response = await client.upsertVectors(mockRequest);
      
      expect(mockedAxios.post).toHaveBeenCalledWith('/v1/vectors/upsert', mockRequest);
      expect(response).toEqual(mockResponse);
    });
    
    test('should handle errors during upsert', async () => {
      const mockRequest: VectorUpsertRequest = {
        collection: 'test-collection',
        provider: 'pinecone',
        vectors: [
          {
            id: 'vec-1',
            values: [0.1, 0.2, 0.3],
            metadata: { source: 'test' }
          }
        ]
      };
      
      const errorResponse = {
        response: {
          status: 400,
          data: {
            error: {
              message: 'Invalid vector format'
            }
          }
        }
      };
      
      mockedAxios.post.mockRejectedValueOnce(errorResponse);
      mockedAxios.isAxiosError.mockReturnValueOnce(true);
      
      await expect(client.upsertVectors(mockRequest))
        .rejects.toThrow('Proksi API Error [400]: Invalid vector format');
    });
  });

  describe('searchVectors', () => {
    test('should successfully search vectors', async () => {
      const mockRequest: VectorSearchRequest = {
        collection: 'test-collection',
        provider: 'pinecone',
        vector: [0.1, 0.2, 0.3],
        top_k: 2,
        filter: { source: 'test' }
      };
      
      const mockResponse: VectorSearchResponse = {
        matches: [
          {
            id: 'vec-1',
            score: 0.95,
            metadata: { source: 'test' }
          },
          {
            id: 'vec-2',
            score: 0.85,
            metadata: { source: 'test' }
          }
        ]
      };
      
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });
      
      const response = await client.searchVectors(mockRequest);
      
      expect(mockedAxios.post).toHaveBeenCalledWith('/v1/vectors/search', mockRequest);
      expect(response).toEqual(mockResponse);
    });
    
    test('should handle empty search results', async () => {
      const mockRequest: VectorSearchRequest = {
        collection: 'test-collection',
        provider: 'pinecone',
        vector: [0.1, 0.2, 0.3],
        top_k: 2
      };
      
      const mockResponse: VectorSearchResponse = {
        matches: []
      };
      
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });
      
      const response = await client.searchVectors(mockRequest);
      
      expect(response.matches).toHaveLength(0);
    });
  });

  describe('deleteVectors', () => {
    test('should successfully delete vectors by IDs', async () => {
      const mockRequest = {
        collection: 'test-collection',
        provider: 'pinecone',
        ids: ['vec-1', 'vec-2']
      };
      
      const mockResponse = {
        message: 'Vectors deleted successfully',
        count: 2
      };
      
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });
      
      const response = await client.deleteVectors(mockRequest);
      
      expect(mockedAxios.post).toHaveBeenCalledWith('/v1/vectors/delete', mockRequest);
      expect(response).toEqual(mockResponse);
    });
    
    test('should successfully delete vectors by filter', async () => {
      const mockRequest = {
        collection: 'test-collection',
        provider: 'pinecone',
        filter: { source: 'test' }
      };
      
      const mockResponse = {
        message: 'Vectors deleted successfully',
        count: 5
      };
      
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });
      
      const response = await client.deleteVectors(mockRequest);
      
      expect(mockedAxios.post).toHaveBeenCalledWith('/v1/vectors/delete', mockRequest);
      expect(response).toEqual(mockResponse);
      expect(response.count).toBe(5);
    });
  });
}); 