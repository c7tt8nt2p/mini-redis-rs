use tokio::io::AsyncWriteExt;

pub mod utils;

const TEST_CONNECTION_ADDRESS: &str = "localhost:0";

#[tokio::test]
async fn send_ping_should_return_pong() {
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS).await;
    let (reader, writer) = socket.split();

    utils::server::write_message(writer, "ping").await;
    let response = utils::client::read_message(reader).await;

    assert_eq!(vec![112, 111, 110, 103, 10], response);

    socket.shutdown().await.unwrap();
}

#[tokio::test]
async fn send_ping_with_data_should_return_data() {
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS).await;
    let (reader, writer) = socket.split();

    utils::server::write_message(writer, "ping hello world").await;
    let response = utils::client::read_message(reader).await;

    assert_eq!(
        vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 10],
        response
    );

    socket.shutdown().await.unwrap();
}

#[tokio::test]
async fn send_xxx_should_return_unknown() {
    let mut socket = utils::start_server_client(TEST_CONNECTION_ADDRESS).await;
    let (reader, writer) = socket.split();

    utils::server::write_message(writer, "xxx").await;
    let response = utils::client::read_message(reader).await;

    assert_eq!(vec![117, 110, 107, 110, 111, 119, 110, 10], response);

    socket.shutdown().await.unwrap();
}
