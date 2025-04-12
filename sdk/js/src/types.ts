/**
 * Types for Proksi AI Gateway SDK
 */

// Common types for all providers
export type LLMProvider = 'openai' | 'anthropic' | 'google' | 'azure' | string;

// Common message format
export interface Message {
  role: 'system' | 'user' | 'assistant' | 'function';
  content: string;
  name?: string;
}

// Base completion request
export interface BaseCompletionRequest {
  provider?: LLMProvider;
  model: string;
  messages: Message[];
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  stream?: boolean;
  functions?: any[];
  user?: string;
  metadata?: Record<string, any>;
}

// OpenAI specific options
export interface OpenAIOptions {
  frequency_penalty?: number;
  presence_penalty?: number;
  logit_bias?: Record<string, number>;
  stop?: string | string[];
}

// Anthropic specific options
export interface AnthropicOptions {
  stop_sequences?: string[];
  system?: string;
}

// Google specific options
export interface GoogleOptions {
  candidate_count?: number;
  stop_sequences?: string[];
}

// Azure specific options
export interface AzureOptions {
  api_version?: string;
  deployment_name?: string;
}

// Combined completion request
export type CompletionRequest = BaseCompletionRequest & 
  Partial<OpenAIOptions> & 
  Partial<AnthropicOptions> & 
  Partial<GoogleOptions> & 
  Partial<AzureOptions>;

// Completion response
export interface CompletionResponse {
  id: string;
  provider: LLMProvider;
  model: string;
  created: number;
  message: Message;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  finish_reason: string;
  metadata?: Record<string, any>;
}

// Stream chunk response
export interface StreamCompletionChunk {
  id: string;
  provider: LLMProvider;
  model: string;
  created: number;
  delta: Partial<Message>;
  finish_reason: string | null;
}

// Vector database types
export interface VectorUpsertRequest {
  namespace: string;
  vectors: {
    id: string;
    values: number[];
    metadata?: Record<string, any>;
  }[];
}

export interface VectorSearchRequest {
  namespace: string;
  query_vector: number[];
  top_k: number;
  filter?: Record<string, any>;
}

export interface VectorSearchResult {
  id: string;
  score: number;
  values?: number[];
  metadata?: Record<string, any>;
}

export interface VectorSearchResponse {
  namespace: string;
  results: VectorSearchResult[];
}

export interface VectorDeleteRequest {
  namespace: string;
  ids: string[];
}

// Client configuration
export interface ProksiClientConfig {
  baseUrl: string;
  apiKey?: string;
  defaultProvider?: LLMProvider;
  defaultModel?: string;
  timeout?: number;
  headers?: Record<string, string>;
} 