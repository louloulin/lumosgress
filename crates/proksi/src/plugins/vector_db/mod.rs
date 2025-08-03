use std::{collections::HashMap, sync::Arc};
use anyhow::Result;
use async_trait::async_trait;
use pingora::{
    http::ResponseHeader,
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::{info, debug, error};
use bytes;

use crate::plugins::core::{Plugin, PluginError, PluginStep, PluginMetadata, PluginType};
use crate::proxy_server::HttpResponse;

use crate::proxy_server::https_proxy::RouterContext;

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
trait VectorDbClient: Send + Sync + std::fmt::Debug {
    async fn upsert(&self, vectors: Vec<Vec<f32>>, metadata: Vec<Value>) -> Result<()>;
    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<Value>>;
    #[allow(dead_code)]
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
        let clients = self.clients.lock().await;
        
        if let Some(_client) = clients.get(&client_key) {
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

    #[allow(dead_code)]
    async fn process_request(&self, session: &mut Session, _config_name: &str) -> Result<()> {
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

    #[allow(dead_code)]
    async fn process_response(&self, session: &mut Session, _config_name: &str) -> Result<()> {
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

    #[allow(dead_code)]
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

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 250, // Vector operations should happen after routing and transformations
            plugin_type: PluginType::Native,
            description: "Vector database integration for semantic search and embeddings storage".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/luizfonseca/proksi".to_string()),
        }
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
        info!("Starting VectorDb plugin");
        info!("VectorDb plugin started with provider: {:?}", self.config.provider);
        info!("Endpoint: {}", self.config.endpoint);
        info!("Collection: {}", self.config.collection);
        info!("Dimensions: {}", self.config.dimensions);
        info!("Batch size: {}", self.config.batch_size.unwrap_or(100));
        info!("Search top-k: {}", self.config.search_top_k.unwrap_or(10));

        if let Some(ref api_key_env) = self.config.api_key_env {
            info!("API key environment variable: {}", api_key_env);
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping VectorDb plugin");
        Ok(())
    }
}

#[derive(Debug)]
struct PineconeClient {
    #[allow(dead_code)]
    config: VectorDbConfig,
}

impl PineconeClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for PineconeClient {
    async fn upsert(&self, _vectors: Vec<Vec<f32>>, _metadata: Vec<Value>) -> Result<()> {
        Ok(())
    }
    
    async fn search(&self, _query: Vec<f32>, _top_k: usize) -> Result<Vec<Value>> {
        Ok(vec![])
    }
    
    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct QdrantClient {
    #[allow(dead_code)]
    config: VectorDbConfig,
}

impl QdrantClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for QdrantClient {
    async fn upsert(&self, _vectors: Vec<Vec<f32>>, _metadata: Vec<Value>) -> Result<()> {
        Ok(())
    }
    
    async fn search(&self, _query: Vec<f32>, _top_k: usize) -> Result<Vec<Value>> {
        Ok(vec![])
    }
    
    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct WeaviateClient {
    #[allow(dead_code)]
    config: VectorDbConfig,
}

impl WeaviateClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for WeaviateClient {
    async fn upsert(&self, _vectors: Vec<Vec<f32>>, _metadata: Vec<Value>) -> Result<()> {
        Ok(())
    }
    
    async fn search(&self, _query: Vec<f32>, _top_k: usize) -> Result<Vec<Value>> {
        Ok(vec![])
    }
    
    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct MilvusClient {
    #[allow(dead_code)]
    config: VectorDbConfig,
}

impl MilvusClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for MilvusClient {
    async fn upsert(&self, _vectors: Vec<Vec<f32>>, _metadata: Vec<Value>) -> Result<()> {
        Ok(())
    }
    
    async fn search(&self, _query: Vec<f32>, _top_k: usize) -> Result<Vec<Value>> {
        Ok(vec![])
    }
    
    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct CustomClient {
    #[allow(dead_code)]
    config: VectorDbConfig,
}

impl CustomClient {
    fn new(config: VectorDbConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VectorDbClient for CustomClient {
    async fn upsert(&self, _vectors: Vec<Vec<f32>>, _metadata: Vec<Value>) -> Result<()> {
        Ok(())
    }
    
    async fn search(&self, _query: Vec<f32>, _top_k: usize) -> Result<Vec<Value>> {
        Ok(vec![])
    }
    
    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Ok(())
    }
}

// Remove static instance
// pub static VECTOR_DB: Lazy<VectorDb> = Lazy::new(VectorDb::new);

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::proxy_server::https_proxy::RouterContext;
    
    #[tokio::test]
    async fn test_plugin_creation() {
        let config = VectorDbConfig {
            provider: VectorDbProvider::Pinecone,
            endpoint: "test-endpoint".to_string(),
            api_key_env: None,
            collection: "test-collection".to_string(),
            dimensions: 1536,
            batch_size: None,
            search_top_k: None,
        };
        
        let plugin = VectorDb::with_config(config.clone());
        assert_eq!(plugin.config.provider, VectorDbProvider::Pinecone);
        assert_eq!(plugin.config.collection, "test-collection");
        let clients = plugin.clients.lock().await;
        assert_eq!(clients.len(), 0); // Client map starts empty
    }
    
    // TODO: Update test when session mocking is available
    /*
    #[tokio::test]
    async fn test_handle_request() {
        let config = VectorDbConfig {
            provider: VectorDbProvider::Pinecone,
            endpoint: "test-endpoint".to_string(),
            api_key_env: None,
            collection: "test-collection".to_string(),
            dimensions: 1536,
            batch_size: None,
            search_top_k: None,
        };
        
        let plugin = VectorDb::with_config(config.clone());
        let mut session = Session::new_dummy();
        let mut ctx = RouterContext::default();
        
        let result = plugin.handle_request(PluginStep::Request, &mut session, &mut ctx).await;
        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(!handled);
        assert!(response.is_none());
    }
    */
} 