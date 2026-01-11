use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn new(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hex::encode(hasher.finalize());
        ContentHash(result)
    }

    pub fn from_str(s: &str) -> Self {
        ContentHash(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_path_component(&self) -> String {
        self.0.clone()
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        ContentHash(String::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub hash: ContentHash,
    pub size: u64,
    pub content_type: String,
    pub created_at: u64,
    pub modified_at: u64,
    pub permissions: u32,
}

impl ContentMetadata {
    pub fn new(content: &[u8], content_type: String) -> Self {
        let hash = ContentHash::new(content);
        let size = content.len() as u64;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        ContentMetadata {
            hash,
            size,
            content_type,
            created_at: now,
            modified_at: now,
            permissions: 0o644,
        }
    }

    pub fn with_permissions(mut self, permissions: u32) -> Self {
        self.permissions = permissions;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ContentAddress {
    pub namespace: String,
    pub hash: ContentHash,
}

impl ContentAddress {
    pub fn new(namespace: String, hash: ContentHash) -> Self {
        ContentAddress { namespace, hash }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.hash.as_str())
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() == 2 {
            Some(ContentAddress::new(
                parts[0].to_string(),
                ContentHash::from_str(parts[1]),
            ))
        } else {
            None
        }
    }
}

impl Hash for ContentAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.hash.hash(state);
    }
}

impl PartialEq for ContentAddress {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.hash == other.hash
    }
}

impl Eq for ContentAddress {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContent {
    pub address: ContentAddress,
    pub metadata: ContentMetadata,
    pub content: Arc<Vec<u8>>,
}

impl StoredContent {
    pub fn new(namespace: String, content: Vec<u8>, content_type: String) -> Self {
        let hash = ContentHash::new(&content);
        let metadata = ContentMetadata::new(&content, content_type);
        let address = ContentAddress::new(namespace, hash);
        StoredContent {
            address,
            metadata,
            content: Arc::new(content),
        }
    }

    pub fn from_bytes(namespace: String, data: Vec<u8>, content_type: String) -> Self {
        StoredContent::new(namespace, data, content_type)
    }

    pub fn from_string(namespace: String, s: String, content_type: String) -> Self {
        StoredContent::from_bytes(namespace, s.into_bytes(), content_type)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.content
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.content).ok()
    }
}

pub struct ContentStore {
    content_index: HashMap<ContentAddress, Arc<StoredContent>>,
    namespace_index: HashMap<String, HashSet<ContentHash>>,
    deduplication_index: HashMap<ContentHash, Arc<StoredContent>>,
    base_path: PathBuf,
}

impl ContentStore {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).ok();
        ContentStore {
            content_index: HashMap::new(),
            namespace_index: HashMap::new(),
            deduplication_index: HashMap::new(),
            base_path,
        }
    }

    pub fn in_memory() -> Self {
        ContentStore {
            content_index: HashMap::new(),
            namespace_index: HashMap::new(),
            deduplication_index: HashMap::new(),
            PathBuf::new(),
        }
    }

    pub fn store(&mut self, content: StoredContent) -> Result<ContentAddress, ContentStoreError> {
        let address = content.address.clone();

        self.content_index.insert(address.clone(), Arc::new(content.clone()));

        self.namespace_index
            .entry(address.namespace.clone())
            .or_insert_with(HashSet::new)
            .insert(address.hash.clone());

        self.deduplication_index
            .insert(address.hash.clone(), Arc::new(content));

        Ok(address)
    }

    pub fn store_bytes(
        &mut self,
        namespace: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<ContentAddress, ContentStoreError> {
        let stored = StoredContent::from_bytes(namespace, data, content_type);
        self.store(stored)
    }

    pub fn store_text(
        &mut self,
        namespace: String,
        text: String,
    ) -> Result<ContentAddress, ContentStoreError> {
        self.store_bytes(namespace, text.into_bytes(), "text/plain".to_string())
    }

    pub fn store_json(
        &mut self,
        namespace: String,
        json: &str,
    ) -> Result<ContentAddress, ContentStoreError> {
        self.store_bytes(namespace, json.as_bytes().to_vec(), "application/json".to_string())
    }

    pub fn retrieve(&self, address: &ContentAddress) -> Option<Arc<StoredContent>> {
        self.content_index.get(address).cloned()
    }

    pub fn retrieve_by_hash(&self, hash: &ContentHash) -> Option<Arc<StoredContent>> {
        self.deduplication_index.get(hash).cloned()
    }

    pub fn exists(&self, address: &ContentAddress) -> bool {
        self.content_index.contains_key(address)
    }

    pub fn exists_by_hash(&self, hash: &ContentHash) -> bool {
        self.deduplication_index.contains_key(hash)
    }

    pub fn list_namespace(&self, namespace: &str) -> Vec<ContentHash> {
        self.namespace_index
            .get(namespace)
            .map(|hashes| hashes.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn namespaces(&self) -> Vec<String> {
        self.namespace_index.keys().cloned().collect()
    }

    pub fn deduplicate(&self, address: &ContentAddress) -> bool {
        self.content_index.contains_key(address)
    }

    pub fn deduplication_stats(&self) -> (usize, usize) {
        let unique = self.deduplication_index.len();
        let total = self.content_index.len();
        (unique, total)
    }

    pub fn persist(&self) -> Result<(), ContentStoreError> {
        if self.base_path.as_os_str().is_empty() {
            return Err(ContentStoreError::NoBasePath);
        }

        let index_path = self.base_path.join("index.json");
        let index_data = serde_json::to_vec(&self.namespace_index)?;
        std::fs::write(index_path, index_data)?;

        for (address, content) in &self.content_index {
            let path = self.path_for_address(address);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content.as_bytes())?;
        }

        Ok(())
    }

    fn path_for_address(&self, address: &ContentAddress) -> PathBuf {
        let mut path = self.base_path.clone();
        path.push(&address.namespace);
        let hash_str = address.hash.to_path_component();
        path.push(&hash_str[..2]);
        path.push(&hash_str[2..]);
        path
    }

    pub fn load(&mut self) -> Result<(), ContentStoreError> {
        if self.base_path.as_os_str().is_empty() {
            return Err(ContentStoreError::NoBasePath);
        }

        let index_path = self.base_path.join("index.json");
        if index_path.exists() {
            let index_data = std::fs::read(index_path)?;
            let namespace_index: HashMap<String, HashSet<ContentHash>> =
                serde_json::from_slice(&index_data)?;
            self.namespace_index = namespace_index;
        }

        for namespace in self.namespaces() {
            for hash in self.list_namespace(&namespace) {
                let address = ContentAddress::new(namespace.clone(), hash);
                let path = self.path_for_address(&address);
                if path.exists() {
                    let content = std::fs::read(path)?;
                    let stored = StoredContent::from_bytes(
                        namespace.clone(),
                        content,
                        "application/octet-stream".to_string(),
                    );
                    self.content_index.insert(address, Arc::new(stored));
                    self.deduplication_index
                        .insert(stored.address.hash.clone(), Arc::new(stored));
                }
            }
        }

        Ok(())
    }

    pub fn gc(&mut self) -> Vec<ContentAddress> {
        let mut referenced = HashSet::new();
        for content in self.content_index.values() {
            referenced.insert(content.address.hash.clone());
        }

        let mut removed = Vec::new();
        self.deduplication_index.retain(|hash, _| {
            let keep = referenced.contains(hash);
            if !keep {
                removed.push(hash.clone());
            }
            keep
        });

        removed
    }
}

#[derive(Debug)]
pub enum ContentStoreError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    NoBasePath,
    ContentNotFound,
}

impl std::fmt::Display for ContentStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentStoreError::Io(e) => write!(f, "IO error: {}", e),
            ContentStoreError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ContentStoreError::NoBasePath => write!(f, "No base path configured"),
            ContentStoreError::ContentNotFound => write!(f, "Content not found"),
        }
    }
}

impl std::error::Error for ContentStoreError {}

impl From<std::io::Error> for ContentStoreError {
    fn from(e: std::io::Error) -> Self {
        ContentStoreError::Io(e)
    }
}

impl From<serde_json::Error> for ContentStoreError {
    fn from(e: serde_json::Error) -> Self {
        ContentStoreError::Serialization(e)
    }
}

pub struct ContentCache {
    store: Arc<parking_lot::RwLock<ContentStore>>,
    cache_size: usize,
    max_cache_size: usize,
    access_order: Vec<ContentHash>,
}

impl ContentCache {
    pub fn new(store: Arc<parking_lot::RwLock<ContentStore>>, max_cache_size: usize) -> Self {
        ContentCache {
            store,
            cache_size: 0,
            max_cache_size,
            access_order: Vec::new(),
        }
    }

    pub fn get(&mut self, hash: &ContentHash) -> Option<Arc<StoredContent>> {
        if let Some(content) = self.store.read().retrieve_by_hash(hash) {
            self.access_order.retain(|h| h != hash);
            self.access_order.push(hash.clone());
            Some(content)
        } else {
            None
        }
    }

    pub fn insert(&mut self, content: Arc<StoredContent>) {
        self.store.write().store_bytes(
            content.address.namespace.clone(),
            content.as_bytes().to_vec(),
            content.metadata.content_type.clone(),
        ).ok();

        self.cache_size += content.as_bytes().len();
        self.access_order.push(content.address.hash.clone());

        while self.cache_size > self.max_cache_size {
            if let Some(oldest) = self.access_order.first() {
                self.access_order.remove(0);
                if let Some(old_content) = self.store.read().retrieve_by_hash(oldest) {
                    self.cache_size -= old_content.as_bytes().len();
                }
            } else {
                break;
            }
        }
    }

    pub fn clear(&mut self) {
        self.access_order.clear();
        self.cache_size = 0;
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let (unique, total) = self.store.read().deduplication_stats();
        (unique, total, self.cache_size)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentManifest {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<ContentAddress>,
    pub content_types: HashMap<String, String>,
    pub checksum: ContentHash,
}

impl ContentManifest {
    pub fn new(name: String, version: String) -> Self {
        ContentManifest {
            name,
            version,
            dependencies: Vec::new(),
            content_types: HashMap::new(),
            checksum: ContentHash::new(&[]),
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<ContentAddress>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_content_types(mut self, types: HashMap<String, String>) -> Self {
        self.content_types = types;
        self
    }

    pub fn compute_checksum(&mut self) {
        let data = serde_json::to_vec(self).unwrap();
        self.checksum = ContentHash::new(&data);
    }
}

pub struct DistributedContentStore {
    local_store: ContentStore,
    network_cache: HashMap<ContentAddress, Arc<StoredContent>>,
    peers: Vec<String>,
    default_namespace: String,
}

impl DistributedContentStore {
    pub fn new(base_path: PathBuf, default_namespace: String) -> Self {
        DistributedContentStore {
            local_store: ContentStore::new(base_path),
            network_cache: HashMap::new(),
            peers: Vec::new(),
            default_namespace,
        }
    }

    pub fn add_peer(&mut self, peer: String) {
        self.peers.push(peer);
    }

    pub fn fetch_from_peer(&self, _peer: &str, _address: &ContentAddress) -> Option<Arc<StoredContent>> {
        None
    }

    pub fn store(&mut self, content: StoredContent) -> Result<ContentAddress, ContentStoreError> {
        self.local_store.store(content)
    }

    pub fn retrieve(&self, address: &ContentAddress) -> Option<Arc<StoredContent>> {
        if let Some(content) = self.local_store.retrieve(address) {
            return Some(content);
        }

        if let Some(content) = self.network_cache.get(address) {
            return Some(content.clone());
        }

        for peer in &self.peers {
            if let Some(content) = self.fetch_from_peer(peer, address) {
                return Some(content);
            }
        }

        None
    }

    pub fn fetch_or_store(&mut self, content: StoredContent) -> Result<ContentAddress, ContentStoreError> {
        if let Some(existing) = self.local_store.retrieve_by_hash(&content.address.hash) {
            return Ok(existing.address.clone());
        }

        self.store(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash() {
        let data = b"hello world";
        let hash1 = ContentHash::new(data);
        let hash2 = ContentHash::new(data);
        assert_eq!(hash1, hash2);

        let different_data = b"hello world!";
        let hash3 = ContentHash::new(different_data);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_content_address() {
        let address = ContentAddress::new("test".to_string(), ContentHash::from_str("abc123"));
        let s = address.to_string();
        assert_eq!(s, "test:abc123");

        let parsed = ContentAddress::from_string(&s).unwrap();
        assert_eq!(parsed.namespace, "test");
        assert_eq!(parsed.hash.as_str(), "abc123");
    }

    #[test]
    fn test_content_store() {
        let mut store = ContentStore::in_memory();

        let content = StoredContent::from_string(
            "test".to_string(),
            "Hello, World!".to_string(),
            "text/plain".to_string(),
        );
        let address = store.store(content).unwrap();

        let retrieved = store.retrieve(&address).unwrap();
        assert_eq!(retrieved.as_str(), Some("Hello, World!"));

        let by_hash = store.retrieve_by_hash(&address.hash).unwrap();
        assert_eq!(by_hash.as_str(), Some("Hello, World!"));
    }

    #[test]
    fn test_deduplication() {
        let mut store = ContentStore::in_memory();

        let content1 = StoredContent::from_string(
            "test".to_string(),
            "same content".to_string(),
            "text/plain".to_string(),
        );
        let addr1 = store.store(content1).unwrap();

        let content2 = StoredContent::from_string(
            "test".to_string(),
            "same content".to_string(),
            "text/plain".to_string(),
        );
        let addr2 = store.store(content2).unwrap();

        assert_eq!(addr1.hash, addr2.hash);

        let (unique, total) = store.deduplication_stats();
        assert_eq!(unique, 1);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_content_manifest() {
        let mut manifest = ContentManifest::new("test-package".to_string(), "1.0.0".to_string());
        manifest.compute_checksum();

        assert!(!manifest.checksum.as_str().is_empty());
    }
}
