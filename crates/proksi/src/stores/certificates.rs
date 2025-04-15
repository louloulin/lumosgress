use openssl::x509::X509;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;

use super::{Store, StoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub cert: Arc<X509>,
    pub key: String,
    pub domains: Vec<String>,
}

pub struct CertificateStore {
    store: super::StorageMap<String, Certificate>,
}

impl CertificateStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(dashmap::DashMap::new()),
        }
    }
}

impl Store<String, Certificate> for CertificateStore {
    fn get(&self, key: &String) -> Option<Certificate> {
        self.store.get(key).map(|v| v.clone())
    }

    fn set(&self, key: String, value: Certificate) -> Result<()> {
        self.store.insert(key, value);
        Ok(())
    }

    fn remove(&self, key: &String) -> Result<()> {
        self.store.remove(key).ok_or(StoreError::NotFound)?;
        Ok(())
    }
}
