import axios from 'axios';
import { ProksiClient } from '../client';
import { CompletionResponse } from '../types';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('ProksiClient', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Setup axios create mock
    mockedAxios.create.mockReturnValue(mockedAxios as any);
  });

  describe('constructor', () => {
    test('should initialize with default settings', () => {
      const client = new ProksiClient({ baseUrl: 'https://api.example.com' });
      
      expect(mockedAxios.create).toHaveBeenCalledWith({
        baseURL: 'https://api.example.com',
        timeout: 30000,
        headers: {
          'Content-Type': 'application/json'
        }
      });
    });

    test('should initialize with custom settings', () => {
      const client = new ProksiClient({
        baseUrl: 'https://api.example.com',
        apiKey: 'test-api-key',
        timeout: 60000,
        headers: { 'X-Custom-Header': 'test' }
      });
      
      expect(mockedAxios.create).toHaveBeenCalledWith({
        baseURL: 'https://api.example.com',
        timeout: 60000,
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer test-api-key',
          'X-Custom-Header': 'test'
        }
      });
    });
  });

  describe('completion', () => {
    test('should send a completion request and return the response', async () => {
      const mockResponse: CompletionResponse = {
        id: 'resp-123',
        provider: 'openai',
        model: 'gpt-4',
        created: Date.now(),
        message: {
          role: 'assistant',
          content: 'This is a test response.'
        },
        usage: {
          prompt_tokens: 10,
          completion_tokens: 5,
          total_tokens: 15
        },
        finish_reason: 'stop'
      };
      
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });
      
      const client = new ProksiClient({
        baseUrl: 'https://api.example.com',
        defaultProvider: 'openai',
        defaultModel: 'gpt-4'
      });
      
      const response = await client.completion({
        messages: [{ role: 'user', content: 'Test' }]
      });
      
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/v1/completions',
        {
          provider: 'openai',
          model: 'gpt-4',
          messages: [{ role: 'user', content: 'Test' }]
        }
      );
      
      expect(response).toEqual(mockResponse);
    });
  });

  describe('error handling', () => {
    test('should enhance axios errors with API details', async () => {
      const errorResponse = {
        response: {
          status: 400,
          data: {
            error: {
              message: 'Invalid request parameters'
            }
          }
        }
      };
      
      mockedAxios.post.mockRejectedValueOnce(errorResponse);
      mockedAxios.isAxiosError.mockReturnValueOnce(true);
      
      const client = new ProksiClient({ baseUrl: 'https://api.example.com' });
      
      await expect(client.completion({
        model: 'gpt-4',
        messages: [{ role: 'user', content: 'Test' }]
      })).rejects.toThrow('Proksi API Error [400]: Invalid request parameters');
    });
  });
}); 