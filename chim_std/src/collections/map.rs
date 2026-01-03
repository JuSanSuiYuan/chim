//! 哈希表实现 - 占位符

use std::collections::HashMap as StdHashMap;

/// 哈希表（临时使用std HashMap）
pub struct Map<K, V> {
    inner: StdHashMap<K, V>,
}

impl<K, V> Map<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            inner: StdHashMap::new(),
        }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }
    
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }
    
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }
    
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.inner.keys()
    }
    
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.values()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }
}

impl<K, V> Default for Map<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
