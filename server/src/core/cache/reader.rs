use std::collections::HashMap;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio::{fs, io};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CacheReaderService: Send + Sync {
    async fn read(&self) -> io::Result<HashMap<String, Vec<u8>>>;
}

pub struct MyCacheReader {
    folder: String,
}

impl MyCacheReader {
    pub fn new(folder: &str) -> Self {
        Self {
            folder: folder.to_owned(),
        }
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

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    use crate::core::cache::reader::{CacheReaderService, MyCacheReader};

    fn create_temp_folder() -> TempDir {
        TempDir::new("cache-reader-tests").unwrap()
    }

    async fn write_data_to_file(temp_dir: &TempDir, file_name: &str, data: Vec<u8>) {
        let temp_file = temp_dir.path().join(file_name);
        let mut file = File::create(temp_file).await.unwrap();
        file.write_all(&data).await.unwrap();
    }

    fn new_instance(temp_dir: &TempDir) -> MyCacheReader {
        let temp_dir_path = temp_dir.path().display().to_string();
        MyCacheReader::new(temp_dir_path.as_str())
    }

    #[tokio::test]
    async fn read_should_be_red() {
        let temp_dir = create_temp_folder();
        write_data_to_file(&temp_dir, "Joe", vec![2u8, 4u8, 6u8, 8u8, 10u8]).await;
        let instance = new_instance(&temp_dir);

        let result = instance.read().await.unwrap();
        assert_eq!(result.len(), 1);
        let joe = result.get_key_value("Joe");
        assert!(joe.is_some());
        assert_eq!(
            joe.unwrap(),
            (&"Joe".to_owned(), &vec![2u8, 4u8, 6u8, 8u8, 10u8])
        );
    }
}
