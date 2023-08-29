pub mod utils;

mod pub_sub {
    use crate::utils::{TEST_CONNECTION_HOST, TEST_CONNECTION_PORT};

    use super::utils;
    use super::utils::client as client_utils;
    use super::utils::file as file_utils;
    use super::utils::server as server_utils;

    #[tokio::test]
    async fn publish_message_to_all_subscribers() {
        let temp_dir = file_utils::create_temp_folder();
        let port = utils::start_server(TEST_CONNECTION_HOST, TEST_CONNECTION_PORT, &temp_dir).await;
        let mut client1 = utils::start_client(port).await;
        let mut client2 = utils::start_client(port).await;
        let (mut reader1, mut writer1) = client1.split();
        let (mut reader2, mut writer2) = client2.split();

        // client1 subscribes
        server_utils::write_message(&mut writer1, "subscribe topicA").await;
        let subscribed_ok_response = client_utils::read_message(&mut reader1).await;
        assert_eq!(
            subscribed_ok_response,
            vec![115, 117, 98, 115, 99, 114, 105, 98, 101, 100, 32, 111, 107, 10]
        );

        // client2 subscribes
        server_utils::write_message(&mut writer2, "subscribe topicA").await;
        let subscribed_ok_response = client_utils::read_message(&mut reader2).await;
        assert_eq!(
            subscribed_ok_response,
            vec![115, 117, 98, 115, 99, 114, 105, 98, 101, 100, 32, 111, 107, 10]
        );

        // client1 publishes a message, client2 reads
        server_utils::write_message(&mut writer1, "hello there").await;
        let message = client_utils::read_message(&mut reader2).await;
        assert_eq!(
            message,
            vec![104, 101, 108, 108, 111, 32, 116, 104, 101, 114, 101]
        );

        // client2 publishes a message, client1 reads
        server_utils::write_message(&mut writer2, "hi there").await;
        let message = client_utils::read_message(&mut reader1).await;
        assert_eq!(message, vec![104, 105, 32, 116, 104, 101, 114, 101]);
    }

    #[tokio::test]
    async fn unsubscribe() {
        let temp_dir = file_utils::create_temp_folder();
        let port = utils::start_server(TEST_CONNECTION_HOST, TEST_CONNECTION_PORT, &temp_dir).await;
        let mut client1 = utils::start_client(port).await;
        let (mut reader1, mut writer1) = client1.split();

        // subscribes
        server_utils::write_message(&mut writer1, "subscribe topicA").await;
        let subscribed_ok_response = client_utils::read_message(&mut reader1).await;
        assert_eq!(
            subscribed_ok_response,
            vec![115, 117, 98, 115, 99, 114, 105, 98, 101, 100, 32, 111, 107, 10]
        );

        // unsubscribes
        server_utils::write_message(&mut writer1, "unsubscribe").await;
        let subscribed_ok_response = client_utils::read_message(&mut reader1).await;
        assert_eq!(
            subscribed_ok_response,
            vec![117, 98, 110, 115, 117, 98, 115, 99, 114, 105, 98, 101, 100, 32, 111, 107, 10]
        );
    }
}
