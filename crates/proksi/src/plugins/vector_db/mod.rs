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
use tracing::{info, warn, debug, error};
use bytes;

use crate::plugins::core::{Plugin, PluginError, PluginStep};
use crate::proxy_server::HttpResponse;

use crate::{
    config::RoutePlugin,
    plugins::get_required_config,
    proxy_server::https_proxy::RouterContext,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorDbConfig {
    pub provider: VectorDbProvider,
    pub endpoint: String,
    pub api_key_env: Option<String>,
    pub collection: String,
    pub dimensions: usize,
    pub batch_size: Option<usize>,
    pub search_top_k: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum VectorDbProvider {
    #[default]
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

#[derive(Debug, Clone)]
pub struct VectorDb {
    config: VectorDbConfig,
    clients: Arc<Mutex<HashMap<String, Box<dyn VectorDbClient>>>>,
}

impl VectorDb {
    pub fn new() -> Self {
        Self {
            config: VectorDbConfig::default(),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(config: VectorDbConfig) -> Self {
        Self {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get_client(&self) -> Result<Box<dyn VectorDbClient>> {
        let client_key = format!("{:?}_{}", self.config.provider, self.config.collection);
        let mut clients = self.clients.lock().await;
        
        if let Some(client) = clients.get(&client_key) {
            // Need a way to return a clone or handle the lifetime
            // For simplicity now, we recreate if not found, but caching is ideal
            // To return from cache directly, client needs to impl Clone or we store Arc<Client>
            // Let's recreate for now to avoid Clone requirement on trait object
            // return Ok(client.clone()); // This won't work directly
        }

        let client: Box<dyn VectorDbClient> = match &self.config.provider {
            VectorDbProvider::Pinecone => Box::new(PineconeClient::new(self.config.clone())),
            VectorDbProvider::Qdrant => Box::new(QdrantClient::new(self.config.clone())),
            VectorDbProvider::Weaviate => Box::new(WeaviateClient::new(self.config.clone())),
            VectorDbProvider::Milvus => Box::new(MilvusClient::new(self.config.clone())),
            VectorDbProvider::Custom(_) => Box::new(CustomClient::new(self.config.clone())),
        };
        
        // Optional: Cache the client if desired for future calls within the same plugin instance lifetime
        // clients.insert(client_key.clone(), client.clone()); // Need client to be Clone or store Arc

        Ok(client)
    }

    async fn process_request(&self, session: &mut Session, config_name: &str) -> Result<()> {
        let client = self.get_client().await?;
        
        let vectors = self.extract_vectors(session).await?;
        let metadata = self.extract_metadata(session).await?;

        let config = self.config.clone();
        let batch_size = config.batch_size.unwrap_or(100);
        for chunk in vectors.chunks(batch_size) {
            let metadata_chunk = metadata[chunk.len()..].to_vec();
            client.upsert(chunk.to_vec(), metadata_chunk).await?;
        }

        Ok(())
    }

    async fn process_response(&self, session: &mut Session, config_name: &str) -> Result<()> {
        let client = self.get_client().await?;
        let config = self.config.clone();
        
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
impl Plugin for VectorDb {
    fn name(&self) -> &'static str {
        "vector_db"
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        _ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        if step == PluginStep::Request {
            if let Some(content_type) = session.req_header().headers.get("content-type") {
                if content_type.to_str().unwrap_or("").contains("application/json") {
                    match self.get_client().await {
                        Ok(client) => {
                            let vectors = self.extract_vectors(session).await?;
                            let metadata = self.extract_metadata(session).await?;
                            let batch_size = self.config.batch_size.unwrap_or(100);
                            
                            info!(provider=?self.config.provider, collection=%self.config.collection, batch_size=batch_size, num_vectors=vectors.len(), "VectorDb: Attempting upsert (using dummy data).");

                            for (vector_chunk, metadata_chunk) in vectors.chunks(batch_size).zip(metadata.chunks(batch_size)) {
                                match client.upsert(vector_chunk.to_vec(), metadata_chunk.to_vec()).await {
                                     Ok(_) => debug!("VectorDb: Batch upsert successful for {} vectors.", vector_chunk.len()),
                                     Err(e) => error!("VectorDb: Batch upsert failed: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            error!("VectorDb: Failed to get client: {}", e);
                        }
                    }
                } else {
                    debug!("VectorDb: Skipping request processing, not JSON.");
                }
            } else {
                 debug!("VectorDb: Skipping request processing, no Content-Type.");
            }
        }
        Ok((false, None))
    }

    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        _ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        if step == PluginStep::Response {
            match self.get_client().await {
                 Ok(client) => {
                     let query_vector = self.extract_query_vector(session).await?;
                     let top_k = self.config.search_top_k.unwrap_or(10);
                     
                     info!(provider=?self.config.provider, collection=%self.config.collection, top_k=top_k, "VectorDb: Attempting search (using dummy data).");

                     match client.search(query_vector, top_k).await {
                         Ok(results) => {
                            info!(num_results = results.len(), "VectorDb: Search successful.");
                            upstream_response.insert_header("X-VectorDB-Search-Performed", "true")?;
                            return Ok(true);
                         }
                         Err(e) => {
                             error!("VectorDb: Search failed: {}", e);
                         }
                     }
                 }
                 Err(e) => {
                     error!("VectorDb: Failed to get client for search: {}", e);
                 }
            }
        }
        Ok(false)
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
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

// Remove static instance
// pub static VECTOR_DB: Lazy<VectorDb> = Lazy::new(VectorDb::new);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::core::PluginStep;
    use crate::proxy_server::https_proxy::RouterContext;
    use pingora::proxy::Session;
    use http::HeaderValue;
    use std::collections::HashMap;
    use serde_json::Value;
    
    // Helper to create a basic RouterContext
    fn create_test_context() -> RouterContext {
        RouterContext {
            host: "test.example.com".to_string(),
            route_container: Default::default(),
            upstream: Default::default(),
            extensions: HashMap::new(),
            is_websocket: false,
            timings: Default::default(),
            upstream_response: None,
            plugins_data: HashMap::new(),
            request_id: String::new(),
        }
    }
    
    #[test]
    fn test_provider_from_str() {
        assert_eq!(VectorDbProvider::from_str("pinecone"), Some(VectorDbProvider::Pinecone));
        assert_eq!(VectorDbProvider::from_str("qdrant"), Some(VectorDbProvider::Qdrant));
        assert_eq!(VectorDbProvider::from_str("WEAVIATE"), Some(VectorDbProvider::Weaviate));
        assert_eq!(VectorDbProvider::from_str("other"), Some(VectorDbProvider::Custom("other".to_string())));
    }
    
    #[test]
    fn test_plugin_creation_with_config() {
        let config = VectorDbConfig {
            provider: VectorDbProvider::Pinecone,
            endpoint: "test-endpoint".to_string(),
            collection: "test-collection".to_string(),
            dimensions: 1536,
            ..Default::default()
        };
        
        let plugin = VectorDb::with_config(config.clone());
        assert_eq!(plugin.config.provider, VectorDbProvider::Pinecone);
        assert_eq!(plugin.config.collection, "test-collection");
        assert_eq!(plugin.clients.lock().unwrap().len(), 0); // Client map starts empty
    }
    
    #[tokio::test]
    async fn test_handle_response_adds_header() {
        let config = VectorDbConfig {
            provider: VectorDbProvider::Pinecone,
            endpoint: "test-endpoint".to_string(),
            collection: "test-collection".to_string(),
            dimensions: 1536,
             search_top_k: Some(5),
            ..Default::default()
        };
        let plugin = VectorDb::with_config(config);
        let mut session = Session::new_dummy();
        let mut ctx = create_test_context();
        let mut upstream_response = ResponseHeader::build(200, Some(4)).unwrap();
        
        // Simulate the response step
        let result = plugin.handle_response(PluginStep::Response, &mut session, &mut ctx, &mut upstream_response).await;
        assert!(result.is_ok());
        // This should be true because the search (even dummy) adds a header
        assert!(result.unwrap()); 
        
        assert!(upstream_response.headers.get("X-VectorDB-Search-Performed").is_some());
        assert_eq!(upstream_response.headers.get("X-VectorDB-Search-Performed").unwrap().to_str().unwrap(), "true");
    }
    
    // NOTE: Testing the core request/response body interactions requires 
    // significant changes to the core plugin execution model or framework capabilities.
} 