use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::cache::reader::CacheReaderService;
use crate::core::cache::writer::CacheWriterService;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait RedisService: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;

    async fn set(&self, key: String, value: Vec<u8>);

    async fn remove(&self, key: &str);

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
        self.db.read().unwrap().get(key).cloned()
    }

    async fn set(&self, key: String, value: Vec<u8>) {
        self.db.write().unwrap().insert(key, value);
    }

    async fn remove(&self, key: &str) {
        self.db.write().unwrap().remove(key);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use mockall::predicate::eq;

    use crate::core::cache::reader::MockCacheReaderService;
    use crate::core::cache::writer::MockCacheWriterService;
    use crate::core::redis::{MyRedisService, RedisService};

    fn mock_deps() -> (MockCacheReaderService, MockCacheWriterService) {
        (MockCacheReaderService::new(), MockCacheWriterService::new())
    }

    fn new_instance(
        mock_cache_reader_service: Arc<MockCacheReaderService>,
        mock_cache_writer_service: Arc<MockCacheWriterService>,
    ) -> MyRedisService {
        MyRedisService::new(mock_cache_reader_service, mock_cache_writer_service)
    }

    #[tokio::test]
    async fn get_should_be_returned() {
        let (cache_reader_service, cache_writer_service) = mock_deps();
        let instance = new_instance(
            Arc::new(cache_reader_service),
            Arc::new(cache_writer_service),
        );

        instance
            .db
            .write()
            .unwrap()
            .insert("hello".to_owned(), vec![111, 112, 113]);

        let x = instance.get("hello");
        let result = x.await;
        assert_eq!(result, Some(vec![111, 112, 113]));
    }

    #[tokio::test]
    async fn set_should_be_set() {
        let (cache_reader_service, cache_writer_service) = mock_deps();
        let instance = new_instance(
            Arc::new(cache_reader_service),
            Arc::new(cache_writer_service),
        );

        instance.set("hi".to_owned(), vec![100, 102, 104]).await;

        let result = instance.db.read().unwrap().get("hi").cloned();
        assert_eq!(result, Some(vec![100, 102, 104]));
    }

    #[tokio::test]
    async fn remove_should_be_remove() {
        let (cache_reader_service, cache_writer_service) = mock_deps();
        let instance = new_instance(
            Arc::new(cache_reader_service),
            Arc::new(cache_writer_service),
        );

        instance
            .db
            .write()
            .unwrap()
            .insert("john".to_owned(), vec![123, 124, 125]);

        let result = instance.db.write().unwrap().remove("john");
        assert_eq!(result, Some(vec![123, 124, 125]));
        assert!(instance.db.read().unwrap().is_empty());
    }

    #[tokio::test]
    async fn read_cache_should_be_red() {
        let (mut cache_reader_service, cache_writer_service) = mock_deps();
        cache_reader_service
            .expect_read()
            .once()
            .returning(|| Ok(HashMap::from([("Jack".to_owned(), vec![111u8, 112u8])])));

        let instance = new_instance(
            Arc::new(cache_reader_service),
            Arc::new(cache_writer_service),
        );

        let result = instance.read_cache().await;

        assert!(result.is_ok());
        let jack = instance.db.read().unwrap().get("Jack").cloned();
        assert_eq!(jack, Some(vec![111u8, 112u8]));
    }

    #[tokio::test]
    async fn write_cache_should_be_written() {
        let (cache_reader_service, mut cache_writer_service) = mock_deps();
        cache_writer_service
            .expect_write()
            .with(eq("John".to_owned()), eq(vec![220u8, 221u8, 222u8]))
            .once()
            .returning(|_, _| Ok(()));

        let instance = new_instance(
            Arc::new(cache_reader_service),
            Arc::new(cache_writer_service),
        );

        let result = instance
            .write_cache("John".to_owned(), vec![220u8, 221u8, 222u8])
            .await;

        assert!(result.is_ok());
    }
}
