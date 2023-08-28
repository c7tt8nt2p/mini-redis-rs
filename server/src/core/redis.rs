#[cfg(test)]
use mockall::{automock, mock, predicate::*};

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::core::cache::reader::CacheReaderService;
use crate::core::cache::writer::CacheWriterService;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait RedisService: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;

    async fn set(&self, key: String, value: Vec<u8>);

    async fn remove(&self, key: &str);

    async fn exists_by_key(&self, key: &str) -> bool;

    async fn read_cache(&self) -> io::Result<()>;

    async fn write_cache(&self, key: String, value: Vec<u8>) -> io::Result<()>;
}

pub struct MyRedisService {
    cache_reader_service: Arc<dyn CacheReaderService>,
    cache_writer_service: Arc<dyn CacheWriterService>,
    db: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MyRedisService {
    pub fn new(
        cache_reader_service: Arc<dyn CacheReaderService>,
        cache_writer_service: Arc<dyn CacheWriterService>,
    ) -> Self {
        Self {
            cache_reader_service,
            cache_writer_service,
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

    async fn remove(&self, key: &str) {
        self.db.write().await.remove(key);
    }

    async fn exists_by_key(&self, key: &str) -> bool {
        self.db.read().await.contains_key(key)
    }

    async fn read_cache(&self) -> io::Result<()> {
        let cache = self.cache_reader_service.read().await?;
        for (key, value) in cache.into_iter() {
            self.set(key, value).await;
        }
        Ok(())
    }

    async fn write_cache(&self, key: String, value: Vec<u8>) -> io::Result<()> {
        self.cache_writer_service.write(key, value).await
    }
}
