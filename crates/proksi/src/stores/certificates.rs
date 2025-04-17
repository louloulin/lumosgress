use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
use std::sync::Arc;
use anyhow::Result;

use super::Store;

// Custom serialization wrappers for OpenSSL types
mod openssl_serde {
    use openssl::pkey::{PKey, Private};
    use openssl::x509::X509;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    pub fn serialize_pkey<S>(key: &PKey<Private>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let pem = key.private_key_to_pem_pkcs8()
            .map_err(|e| serde::ser::Error::custom(format!("Failed to serialize PKey: {}", e)))?;
        let pem_str = String::from_utf8(pem)
            .map_err(|e| serde::ser::Error::custom(format!("Failed to convert PKey to UTF-8: {}", e)))?;
        serializer.serialize_str(&pem_str)
    }

    pub fn deserialize_pkey<'de, D>(deserializer: D) -> Result<PKey<Private>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pem_str = String::deserialize(deserializer)?;
        PKey::private_key_from_pem(pem_str.as_bytes())
            .map_err(|e| serde::de::Error::custom(format!("Failed to deserialize PKey: {}", e)))
    }

    pub fn serialize_x509<S>(cert: &Arc<X509>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let pem = cert.to_pem()
            .map_err(|e| serde::ser::Error::custom(format!("Failed to serialize X509: {}", e)))?;
        let pem_str = String::from_utf8(pem)
            .map_err(|e| serde::ser::Error::custom(format!("Failed to convert X509 to UTF-8: {}", e)))?;
        serializer.serialize_str(&pem_str)
    }

    pub fn deserialize_x509<'de, D>(deserializer: D) -> Result<Arc<X509>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pem_str = String::deserialize(deserializer)?;
        let cert = X509::from_pem(pem_str.as_bytes())
            .map_err(|e| serde::de::Error::custom(format!("Failed to deserialize X509: {}", e)))?;
        Ok(Arc::new(cert))
    }
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub cert: Arc<X509>,
    pub key: PKey<Private>,
    pub domains: Vec<String>,
    
    // Additional fields to match usage in discovery module
    pub leaf_cert: Option<X509>,
    pub cert_chain: Option<Vec<X509>>,
}

// Helper methods for compatibility with older code
impl Certificate {
    pub fn leaf(&self) -> &X509 {
        self.leaf_cert.as_ref().unwrap_or(&self.cert)
    }
    
    pub fn chain(&self) -> Option<&Vec<X509>> {
        self.cert_chain.as_ref()
    }
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
        self.store.remove(key).ok_or_else(|| anyhow::anyhow!("Key not found"))?;
        Ok(())
    }
}
