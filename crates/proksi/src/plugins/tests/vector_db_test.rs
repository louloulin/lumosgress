#[cfg(test)]
mod vector_db_tests {
    use super::*;
    use crate::plugins::vector_db::{VectorDb, VectorDbConfig, VectorDbProvider};
    use serde_json::json;
    use std::collections::HashMap;
    use pingora::proxy::Session;
    use crate::config::RoutePlugin;
    use crate::proxy_server::https_proxy::RouterContext;

    fn create_test_config() -> VectorDbConfig {
        VectorDbConfig {
            provider: VectorDbProvider::Pinecone,
            endpoint: "https://test-pinecone.svc.io".to_string(),
            api_key_env: Some("PINECONE_API_KEY".to_string()),
            collection: "test-collection".to_string(),
            dimensions: 1536,
            batch_size: Some(100),
            search_top_k: Some(10),
        }
    }

    #[tokio::test]
    async fn test_vector_db_config() {
        let config = create_test_config();
        
        assert!(matches!(config.provider, VectorDbProvider::Pinecone));
        assert_eq!(config.endpoint, "https://test-pinecone.svc.io");
        assert_eq!(config.api_key_env, Some("PINECONE_API_KEY".to_string()));
        assert_eq!(config.collection, "test-collection");
        assert_eq!(config.dimensions, 1536);
        assert_eq!(config.batch_size, Some(100));
        assert_eq!(config.search_top_k, Some(10));
    }

    #[tokio::test]
    async fn test_vector_db_provider_from_str() {
        assert!(matches!(
            VectorDbProvider::from_str("pinecone").unwrap(),
            VectorDbProvider::Pinecone
        ));
        assert!(matches!(
            VectorDbProvider::from_str("qdrant").unwrap(),
            VectorDbProvider::Qdrant
        ));
        assert!(matches!(
            VectorDbProvider::from_str("weaviate").unwrap(),
            VectorDbProvider::Weaviate
        ));
        assert!(matches!(
            VectorDbProvider::from_str("milvus").unwrap(),
            VectorDbProvider::Milvus
        ));
        assert!(matches!(
            VectorDbProvider::from_str("custom").unwrap(),
            VectorDbProvider::Custom(_)
        ));
    }

    #[tokio::test]
    async fn test_vector_db_creation() {
        let vector_db = VectorDb::new();
        
        // 验证VectorDb实例创建成功
        assert!(vector_db.config.is_empty());
        assert!(vector_db.clients.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_vector_extraction() {
        let vector_db = VectorDb::new();
        
        // 创建测试会话
        let mut session = Session::new();
        session.req_body = Some(json!({
            "vectors": [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]],
            "metadata": [{"id": "1"}, {"id": "2"}]
        }).to_string().into_bytes());
        
        // 测试向量提取
        let vectors = vector_db.extract_vectors(&session).unwrap();
        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(vectors[1], vec![0.4, 0.5, 0.6]);
        
        // 测试元数据提取
        let metadata = vector_db.extract_metadata(&session).unwrap();
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata[0]["id"], "1");
        assert_eq!(metadata[1]["id"], "2");
    }

    #[tokio::test]
    async fn test_query_vector_extraction() {
        let vector_db = VectorDb::new();
        
        // 创建测试会话
        let mut session = Session::new();
        session.req_body = Some(json!({
            "query_vector": [0.1, 0.2, 0.3]
        }).to_string().into_bytes());
        
        // 测试查询向量提取
        let query_vector = vector_db.extract_query_vector(&session).unwrap();
        assert_eq!(query_vector, vec![0.1, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn test_search_results_addition() {
        let vector_db = VectorDb::new();
        
        // 创建测试会话
        let mut session = Session::new();
        let results = vec![
            json!({"id": "1", "score": 0.9}),
            json!({"id": "2", "score": 0.8})
        ];
        
        // 测试结果添加
        vector_db.add_search_results(&mut session, results.clone()).await.unwrap();
        
        // 验证响应体
        let response_body = session.resp_body.as_ref().unwrap();
        let expected = json!({
            "search_results": results
        });
        assert_eq!(response_body, &expected.to_string().as_bytes());
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let vector_db = VectorDb::new();
        
        // 创建测试会话
        let mut session = Session::new();
        let vectors = vec![vec![0.1; 1536]; 250]; // 250个向量
        let metadata = vec![json!({"id": "1"}); 250];
        
        session.req_body = Some(json!({
            "vectors": vectors,
            "metadata": metadata
        }).to_string().into_bytes());
        
        // 测试批量处理
        vector_db.process_request(&mut session, "test_config").await.unwrap();
        
        // 验证处理结果
        // 注意：这里需要根据具体的实现来验证
    }

    #[tokio::test]
    async fn test_plugin_integration() {
        let vector_db = VectorDb::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/vector".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: crate::proxy_server::https_proxy::RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        let plugin = RoutePlugin {
            name: "vector_db".to_string(),
            config: Some({
                let mut map = HashMap::new();
                map.insert("config_name".into(), serde_json::Value::String("test_config".to_string()));
                map
            }),
        };
        
        // 测试请求过滤器
        let result = vector_db.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
        
        // 测试响应过滤器
        let result = vector_db.response_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
    }
} 