import axios, { AxiosInstance, AxiosError } from 'axios';
import {
  ProksiClientConfig,
  CompletionRequest,
  CompletionResponse,
  StreamCompletionChunk,
  VectorUpsertRequest,
  VectorSearchRequest,
  VectorSearchResponse,
  VectorDeleteRequest
} from './types';

/**
 * ProksiClient is the main class to interact with the Proksi AI Gateway API
 */
export class ProksiClient {
  private client: AxiosInstance;
  private defaultProvider?: string;
  private defaultModel?: string;

  /**
   * Create a new Proksi API client
   * @param config Configuration for the client
   */
  constructor(config: ProksiClientConfig) {
    const { baseUrl, apiKey, timeout = 30000, headers = {}, defaultProvider, defaultModel } = config;

    // Configure headers
    const defaultHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Add authorization if API key is provided
    if (apiKey) {
      defaultHeaders['Authorization'] = `Bearer ${apiKey}`;
    }

    // Create axios instance
    this.client = axios.create({
      baseURL: baseUrl,
      timeout,
      headers: { ...defaultHeaders, ...headers }
    });

    this.defaultProvider = defaultProvider;
    this.defaultModel = defaultModel;
  }

  /**
   * Send a completion request to the LLM provider
   * @param request Completion request parameters
   * @returns Completion response
   */
  async completion(request: CompletionRequest): Promise<CompletionResponse> {
    try {
      // Apply defaults if not specified in the request
      const fullRequest = {
        ...request
      };
      
      // Only add defaults if not already in the request
      if (!fullRequest.provider && this.defaultProvider) {
        fullRequest.provider = this.defaultProvider;
      }
      
      if (!fullRequest.model && this.defaultModel) {
        fullRequest.model = this.defaultModel;
      }

      const response = await this.client.post<CompletionResponse>('/v1/completions', fullRequest);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Stream a completion request, receiving chunks as they are generated
   * @param request Completion request parameters
   * @param onChunk Callback for each chunk received
   * @param onError Optional error callback
   * @param onDone Optional completion callback
   */
  async streamCompletion(
    request: CompletionRequest,
    onChunk: (chunk: StreamCompletionChunk) => void,
    onError?: (error: Error) => void,
    onDone?: () => void
  ): Promise<void> {
    try {
      // Apply defaults if not specified in the request
      const fullRequest = {
        ...request,
        stream: true
      };
      
      // Only add defaults if not already in the request
      if (!fullRequest.provider && this.defaultProvider) {
        fullRequest.provider = this.defaultProvider;
      }
      
      if (!fullRequest.model && this.defaultModel) {
        fullRequest.model = this.defaultModel;
      }

      const response = await this.client.post('/v1/completions', fullRequest, {
        responseType: 'stream',
        headers: {
          'Accept': 'text/event-stream',
        }
      });

      const stream = response.data;
      let buffer = '';

      stream.on('data', (chunk: Buffer) => {
        const text = chunk.toString();
        buffer += text;

        // Process complete events in the buffer
        const lines = buffer.split('\n\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6);
            if (data === '[DONE]') {
              if (onDone) onDone();
              return;
            }

            try {
              const parsed = JSON.parse(data) as StreamCompletionChunk;
              onChunk(parsed);
            } catch (e) {
              console.error('Error parsing SSE data:', e);
            }
          }
        }
      });

      stream.on('error', (err: Error) => {
        if (onError) onError(err);
      });

      stream.on('end', () => {
        if (onDone) onDone();
      });

    } catch (error) {
      const processedError = this.handleError(error);
      if (onError) onError(processedError);
      else throw processedError;
    }
  }

  /**
   * Upsert vectors into a vector database
   * @param request Vector upsert request parameters
   */
  async upsertVectors(request: VectorUpsertRequest): Promise<void> {
    try {
      await this.client.post('/v1/vectors/upsert', request);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Search for similar vectors in a vector database
   * @param request Vector search request parameters
   * @returns Vector search results
   */
  async searchVectors(request: VectorSearchRequest): Promise<VectorSearchResponse> {
    try {
      const response = await this.client.post<VectorSearchResponse>('/v1/vectors/search', request);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Delete vectors from a vector database
   * @param request Vector deletion request parameters
   */
  async deleteVectors(request: VectorDeleteRequest): Promise<void> {
    try {
      await this.client.post('/v1/vectors/delete', request);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Handle API errors and enhance them with better context
   */
  private handleError(error: unknown): Error {
    if (axios.isAxiosError(error)) {
      const axiosError = error as AxiosError;
      if (axiosError.response) {
        const status = axiosError.response.status;
        const data = axiosError.response.data as any;
        const message = data?.error?.message || axiosError.message;
        return new Error(`Proksi API Error [${status}]: ${message}`);
      }
      return new Error(`Proksi API Error: ${axiosError.message}`);
    }
    return error instanceof Error ? error : new Error(String(error));
  }
} 