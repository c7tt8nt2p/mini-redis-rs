use std::collections::HashMap;

use async_trait::async_trait;
use tokio::{fs, io};

#[async_trait]
pub trait CacheReaderService: Send + Sync {
    async fn read(&self) -> io::Result<HashMap<String, Vec<u8>>>;
}

pub struct MyCacheReader {
    folder: String,
}

impl MyCacheReader {
    pub fn new(folder: String) -> Self {
        Self { folder }
    }
}

#[async_trait]
impl CacheReaderService for MyCacheReader {
    async fn read(&self) -> io::Result<HashMap<String, Vec<u8>>> {
        let mut cache = HashMap::<String, Vec<u8>>::new();
        println!("reading cache... from: {}", self.folder);
        let mut dir = fs::read_dir(&self.folder).await?;
        while let Ok(Some(entry)) = dir.next_entry().await {
            if entry.file_type().await?.is_file() {
                println!("\tuncache: {}", entry.path().to_str().unwrap());
                let file_contents = fs::read(entry.path()).await?;
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap().to_owned();
                cache.insert(file_name, file_contents.clone());
            }
        }
        println!("reading cache... done");
        Ok(cache)
    }
}
