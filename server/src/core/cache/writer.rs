use std::path::Path;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CacheWriterService: Send + Sync {
    async fn write(&self, key: String, value: Vec<u8>) -> io::Result<()>;
}

pub struct MyCacheWriter {
    folder: String,
}

impl MyCacheWriter {
    pub fn new(folder: &str) -> Self {
        Self {
            folder: folder.to_owned(),
        }
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

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use tokio::fs;

    use crate::core::cache::writer::{CacheWriterService, MyCacheWriter};

    fn create_temp_folder() -> TempDir {
        TempDir::new("cache-writer-tests").unwrap()
    }

    fn new_instance(temp_dir: &TempDir) -> MyCacheWriter {
        let temp_dir_path = temp_dir.path().display().to_string();
        MyCacheWriter::new(temp_dir_path.as_str())
    }

    #[tokio::test]
    async fn write_should_be_written() {
        let temp_dir = create_temp_folder();
        let instance = new_instance(&temp_dir);

        let result = instance
            .write("hello".to_owned(), vec![200u8, 201u8, 202u8])
            .await;

        assert!(result.is_ok());
        let cache_file = temp_dir.path().join("hello");
        let is_file = fs::metadata(cache_file.clone()).await.map(|f| f.is_file());
        assert!(is_file.is_ok());
        let file_contents = fs::read(cache_file.clone()).await;
        assert!(file_contents.is_ok());
        assert_eq!(file_contents.unwrap(), vec![200u8, 201u8, 202u8]);
    }
}
