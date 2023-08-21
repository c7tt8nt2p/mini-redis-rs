use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

#[async_trait]
pub trait RedisService: Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;

    async fn set(&self, key: String, value: Vec<u8>);

    async fn exists_by_key(&self, key: &str) -> bool;
}

pub struct MyRedisService {
    db: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MyRedisService {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RedisService for MyRedisService {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.db.read().await.get(key).cloned()
    }

    async fn set(&self, key: String, value: Vec<u8>) {
        self.db.write().await.insert(key, value);
    }

    async fn exists_by_key(&self, key: &str) -> bool {
        self.db.read().await.contains_key(key)
    }
}
