use std::path::Path;

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[async_trait]
pub trait CacheWriterService: Send + Sync {
    async fn write(&self, key: String, value: Vec<u8>) -> io::Result<()>;
}

pub struct MyCacheWriter {
    folder: String,
}

impl MyCacheWriter {
    pub fn new(folder: String) -> Self {
        Self { folder }
    }
}

#[async_trait]
impl CacheWriterService for MyCacheWriter {
    async fn write(&self, key: String, value: Vec<u8>) -> io::Result<()> {
        let file_path = Path::new(&self.folder).join(key);
        let mut cache_file = File::create(file_path).await?;
        cache_file.write_all(&value).await
    }
}
