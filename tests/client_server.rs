use tokio::fs::metadata;

use crate::utils::server::create_temp_folder;

pub mod utils;

const TEST_CONNECTION_ADDRESS: &str = "localhost:0";

#[tokio::test]
async fn send_ping_should_return_pong() {
    let temp_dir = create_temp_folder();
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS, &temp_dir).await;
    let (mut reader, mut writer) = socket.split();

    utils::server::write_message(&mut writer, "ping").await;
    let response = utils::client::read_message(&mut reader).await;

    assert_eq!(vec![112, 111, 110, 103, 10], response);
}

#[tokio::test]
async fn send_ping_with_data_should_return_data() {
    let temp_dir = create_temp_folder();
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS, &temp_dir).await;
    let (mut reader, mut writer) = socket.split();

    utils::server::write_message(&mut writer, "ping hello world").await;
    let response = utils::client::read_message(&mut reader).await;

    assert_eq!(
        vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 10],
        response
    );
}

#[tokio::test]
async fn send_xxx_should_return_unknown() {
    let temp_dir = create_temp_folder();
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS, &temp_dir).await;
    let (mut reader, mut writer) = socket.split();

    utils::server::write_message(&mut writer, "xxx").await;
    let response = utils::client::read_message(&mut reader).await;

    assert_eq!(vec![117, 110, 107, 110, 111, 119, 110, 10], response);
}

#[tokio::test]
async fn set_and_get_should_return_data() {
    let temp_dir = create_temp_folder();
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS, &temp_dir).await;
    let (mut reader, mut writer) = socket.split();

    utils::server::write_message(&mut writer, "set a hello world").await;
    let set_response = utils::client::read_message(&mut reader).await;
    assert_eq!(vec![115, 101, 116, 32, 111, 107, 10], set_response);

    utils::server::write_message(&mut writer, "get a").await;
    let get_response = utils::client::read_message(&mut reader).await;
    assert_eq!(
        vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 10],
        get_response
    );
}

#[tokio::test]
async fn get_from_cache_should_return_data() {
    let temp_dir = create_temp_folder();
    let data: [u8; 11] = [1, 0, 0, 0, 0, 0, 0, 0, 2, 104, 105];
    utils::file::write_data_to_file(&temp_dir, "testcache", &data).await;

    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS, &temp_dir).await;
    let (mut reader, mut writer) = socket.split();

    utils::server::write_message(&mut writer, "get testcache").await;
    let get_response = utils::client::read_message(&mut reader).await;
    assert_eq!(vec![104, 105, 10], get_response);
}

#[tokio::test]
async fn set_should_cache_to_file() {
    let temp_dir = create_temp_folder();
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS, &temp_dir).await;
    let (mut reader, mut writer) = socket.split();

    utils::server::write_message(&mut writer, "set a aa").await;
    let set_response = utils::client::read_message(&mut reader).await;
    assert_eq!(vec![115, 101, 116, 32, 111, 107, 10], set_response);

    let temp_file = temp_dir.path().join("a");
    let file_exists = metadata(temp_file).await.unwrap().is_file();
    assert!(file_exists);
}
