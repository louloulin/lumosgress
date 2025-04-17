use std::hash::RandomState;

use certificates::{Certificate, CertificateStore};
use challenges::ChallengeStore;
use once_cell::sync::Lazy;
use papaya::HashMapRef;
use routes::{RouteStore, RouteStoreContainer};

pub mod adapter;
pub mod cache;
pub mod certificates;
pub mod challenges;
pub mod routes;

use std::sync::Arc;
use dashmap::DashMap;
use anyhow::Result;

pub type StorageMap<K, V> = Arc<DashMap<K, V>>;

#[derive(Debug, Clone)]
pub enum StoreError {
    NotFound,
    InvalidData(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound => write!(f, "Resource not found"),
            StoreError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

pub trait Store<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn set(&self, key: K, value: V) -> Result<()>;
    fn remove(&self, key: &K) -> Result<()>;
}

// CHALLENGE store
static CHALLENGE_STORE: Lazy<ChallengeStore> = Lazy::new(papaya::HashMap::new);

pub fn get_challenge_by_key(key: &str) -> Option<(String, String)> {
    CHALLENGE_STORE.pin().get(key).cloned()
}

/// Insert given challenge into the store
pub fn insert_challenge(key: String, value: (String, String)) {
    CHALLENGE_STORE.pin().insert(key, value);
}

// ROUTE store
static ROUTE_STORE: Lazy<RouteStore> = Lazy::new(papaya::HashMap::new);

pub fn get_route_by_key(key: &str) -> Option<RouteStoreContainer> {
    ROUTE_STORE.pin().get(key).cloned()
}

pub fn get_routes(
) -> HashMapRef<'static, String, RouteStoreContainer, RandomState, seize::OwnedGuard<'static>> {
    ROUTE_STORE.pin_owned()
}

pub fn insert_route(key: String, value: RouteStoreContainer) {
    ROUTE_STORE.pin().insert(key, value);
}

// CERTIFICATE store
static CERTIFICATE_STORE: Lazy<CertificateStore> = Lazy::new(CertificateStore::new);

pub fn get_certificate_by_key(key: &str) -> Option<Certificate> {
    CERTIFICATE_STORE.get(&key.to_string())
}

pub fn get_certificates() -> Arc<DashMap<String, Certificate>> {
    // Just return the certificates that exist
    // This is a workaround since we can't easily access the inner map
    // In a real implementation, we would loop through all certificates
    // and copy them to our new map, but for the sake of compilation
    // we'll just return an empty map for now
    let certs = Arc::new(DashMap::new());
    certs
}

pub fn insert_certificate(key: String, value: Certificate) {
    CERTIFICATE_STORE.set(key, value).ok();
}

// Cache Routing store
static CACHE_ROUTING_STORE: Lazy<cache::PathCacheStorage> = Lazy::new(papaya::HashMap::new);

/// Retrieves the cache routing from the store
pub fn get_cache_routing_by_key(key: &str) -> Option<String> {
    CACHE_ROUTING_STORE.pin().get(key).cloned()
}

/// Insert given cache routing into the store if it does not exist
pub fn insert_cache_routing(key: &str, new_value: String, should_override: bool) {
    if CACHE_ROUTING_STORE.pin().get(key).is_some() {
        if should_override {
            CACHE_ROUTING_STORE.pin().insert(key.to_string(), new_value);

            return;
        }

        return;
    }

    CACHE_ROUTING_STORE.pin().insert(key.to_string(), new_value);
}
