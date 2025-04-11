use std::{borrow::Cow, collections::HashMap, sync::Arc};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use http::HeaderValue;
use once_cell::sync::Lazy;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::{info, warn};
use bytes;

use crate::{
    config::RoutePlugin,
    plugins::get_required_config,
    proxy_server::https_proxy::RouterContext,
};

use super::MiddlewarePlugin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    pub provider: VectorDbProvider,
    pub endpoint: String,
    pub api_key_env: Option<String>,
    pub collection: String,
    pub dimensions: usize,
    pub batch_size: Option<usize>,
    pub search_top_k: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VectorDbProvider {
    Pinecone,
    Qdrant,
    Weaviate,
    Milvus,
    Custom(String),
}

impl VectorDbProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pinecone" => Some(Self::Pinecone),
            "qdrant" => Some(Self::Qdrant),
            "weaviate" => Some(Self::Weaviate),
            "milvus" => Some(Self::Milvus),
            _ => Some(Self::Custom(s.to_string())),
        }
    }
}

#[async_trait]
trait VectorDbClient: Send + Sync {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()>;
    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>>;
    async fn delete(&self, ids: Vec<String>) -> Result<()>;
}

pub struct VectorDb {
    config: Arc<HashMap<String, VectorDbConfig>>,
    clients: Arc<Mutex<HashMap<String, Box<dyn VectorDbClient>>>>,
}

impl VectorDb {
    pub fn new() -> Self {
        Self {
            config: Arc::new(HashMap::new()),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get_client(&self, config_name: &str) -> Result<Box<dyn VectorDbClient>> {
        let clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(config_name) {
            let config = self.config.get(config_name)
                .ok_or_else(|| anyhow!("Vector DB config not found: {}", config_name))?;
            
            return match config.provider {
                VectorDbProvider::Pinecone => Ok(Box::new(PineconeClient::new(config.clone()))),
                VectorDbProvider::Qdrant => Ok(Box::new(QdrantClient::new(config.clone()))),
                VectorDbProvider::Weaviate => Ok(Box::new(WeaviateClient::new(config.clone()))),
                VectorDbProvider::Milvus => Ok(Box::new(MilvusClient::new(config.clone()))),
                VectorDbProvider::Custom(_) => Ok(Box::new(CustomClient::new(config.clone()))),
            };
        }
        
        let config = self.config.get(config_name)
            .ok_or_else(|| anyhow!("Vector DB config not found: {}", config_name))?;
            
        let client: Box<dyn VectorDbClient> = match config.provider {
            VectorDbProvider::Pinecone => Box::new(PineconeClient::new(config.clone())),
            VectorDbProvider::Qdrant => Box::new(QdrantClient::new(config.clone())),
            VectorDbProvider::Weaviate => Box::new(WeaviateClient::new(config.clone())),
            VectorDbProvider::Milvus => Box::new(MilvusClient::new(config.clone())),
            VectorDbProvider::Custom(_) => Box::new(CustomClient::new(config.clone())),
        };
        
        drop(clients);
        
        let mut clients = self.clients.lock().await;
        clients.insert(config_name.to_string(), match config.provider {
            VectorDbProvider::Pinecone => Box::new(PineconeClient::new(config.clone())),
            VectorDbProvider::Qdrant => Box::new(QdrantClient::new(config.clone())),
            VectorDbProvider::Weaviate => Box::new(WeaviateClient::new(config.clone())),
            VectorDbProvider::Milvus => Box::new(MilvusClient::new(config.clone())),
            VectorDbProvider::Custom(_) => Box::new(CustomClient::new(config.clone())),
        });
        
        Ok(client)
    }

    async fn process_request(&self, session: &mut Session, config_name: &str) -> Result<()> {
        let client = self.get_client(config_name).await?;
        
        let vectors = self.extract_vectors(session).await?;
        let metadata = self.extract_metadata(session).await?;

        let config = self.config.get(config_name).unwrap();
        let batch_size = config.batch_size.unwrap_or(100);
        for chunk in vectors.chunks(batch_size) {
            let metadata_chunk = metadata[chunk.len()..].to_vec();
            client.upsert(chunk.to_vec(), metadata_chunk).await?;
        }

        Ok(())
    }

    async fn process_response(&self, session: &mut Session, config_name: &str) -> Result<()> {
        let client = self.get_client(config_name).await?;
        let config = self.config.get(config_name).unwrap();
        
        let query_vector = self.extract_query_vector(session).await?;
        
        let top_k = config.search_top_k.unwrap_or(10);
        let results = client.search(query_vector, top_k).await?;
        
        self.add_search_results(session, results).await?;

        Ok(())
    }

    async fn extract_vectors(&self, session: &mut Session) -> Result<Vec<Vec<f32>>> {
        if let Some(body) = session.read_request_body().await.ok().flatten() {
            if let Ok(json_body) = serde_json::from_slice::<Value>(&body) {
                if let Some(vectors) = json_body.get("vectors").and_then(|v| v.as_array()) {
                    return Ok(vectors.iter()
                        .filter_map(|v| v.as_array())
                        .filter_map(|v| v.iter().filter_map(|f| f.as_f64()).map(|f| f as f32).collect::<Vec<_>>().into())
                        .collect());
                }
            }
        }
        Ok(vec![])
    }

    async fn extract_metadata(&self, session: &mut Session) -> Result<Vec<Value>> {
        if let Some(body) = session.read_request_body().await.ok().flatten() {
            if let Ok(json_body) = serde_json::from_slice::<Value>(&body) {
                if let Some(metadata) = json_body.get("metadata").and_then(|m| m.as_array()) {
                    return Ok(metadata.clone());
                }
            }
        }
        Ok(vec![])
    }

    async fn extract_query_vector(&self, session: &mut Session) -> Result<Vec<f32>> {
        if let Some(body) = session.read_request_body().await.ok().flatten() {
            if let Ok(json_body) = serde_json::from_slice::<Value>(&body) {
                if let Some(vector) = json_body.get("query_vector").and_then(|v| v.as_array()) {
                    return Ok(vector.iter()
                        .filter_map(|f| f.as_f64())
                        .map(|f| f as f32)
                        .collect());
                }
            }
        }
        Ok(vec![])
    }

    async fn add_search_results(&self, session: &mut Session, results: Vec<Value>) -> Result<()> {
        let response = json!({
            "search_results": results
        });
        
        let response_bytes = response.to_string();
        session.write_response_body(Some(bytes::Bytes::from(response_bytes)), true).await?;
        Ok(())
    }
}

#[async_trait]
impl MiddlewarePlugin for VectorDb {
    async fn request_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        let config_data = config.config.as_ref().ok_or_else(|| anyhow!("Vector DB plugin requires configuration"))?;
        let config_name = get_required_config(config_data, "config_name").unwrap_or_else(|_| "default".to_string());
        
        self.process_request(session, &config_name).await?;
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut RequestHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        Ok(())
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        let config_data = config.config.as_ref().ok_or_else(|| anyhow!("Vector DB plugin requires configuration"))?;
        let config_name = get_required_config(config_data, "config_name").unwrap_or_else(|_| "default".to_string());
        
        self.process_response(session, &config_name).await?;
        Ok(false)
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        Ok(())
    }
}

struct PineconeClient {
    config: VectorDbConfig,
}

impl PineconeClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for PineconeClient {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()> {
        // TODO: 实现Pinecone upsert逻辑
        Ok(())
    }

    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>> {
        // TODO: 实现Pinecone search逻辑
        Ok(vec![])
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        // TODO: 实现Pinecone delete逻辑
        Ok(())
    }
}

struct QdrantClient {
    config: VectorDbConfig,
}

impl QdrantClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for QdrantClient {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()> {
        // TODO: 实现Qdrant upsert逻辑
        Ok(())
    }

    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>> {
        // TODO: 实现Qdrant search逻辑
        Ok(vec![])
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        // TODO: 实现Qdrant delete逻辑
        Ok(())
    }
}

struct WeaviateClient {
    config: VectorDbConfig,
}

impl WeaviateClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for WeaviateClient {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()> {
        // TODO: 实现Weaviate upsert逻辑
        Ok(())
    }

    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>> {
        // TODO: 实现Weaviate search逻辑
        Ok(vec![])
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        // TODO: 实现Weaviate delete逻辑
        Ok(())
    }
}

struct MilvusClient {
    config: VectorDbConfig,
}

impl MilvusClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for MilvusClient {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()> {
        // TODO: 实现Milvus upsert逻辑
        Ok(())
    }

    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>> {
        // TODO: 实现Milvus search逻辑
        Ok(vec![])
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        // TODO: 实现Milvus delete逻辑
        Ok(())
    }
}

struct CustomClient {
    config: VectorDbConfig,
}

impl CustomClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for CustomClient {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()> {
        // TODO: 实现自定义向量数据库upsert逻辑
        Ok(())
    }

    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>> {
        // TODO: 实现自定义向量数据库search逻辑
        Ok(vec![])
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        // TODO: 实现自定义向量数据库delete逻辑
        Ok(())
    }
}

pub static VECTOR_DB: Lazy<VectorDb> = Lazy::new(VectorDb::new); 