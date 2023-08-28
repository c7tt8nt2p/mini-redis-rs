use tempdir::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn create_temp_folder() -> TempDir {
    TempDir::new("mini-reds-integration-tests").unwrap()
}

pub async fn write_data_to_file(temp_dir: &TempDir, file_name: &str, data: &[u8]) {
    let temp_file = temp_dir.path().join(file_name);
    let mut file = File::create(temp_file).await.unwrap();
    file.write_all(data).await.unwrap();
}
