use regex::Regex;

const PING_VALUE_REGEX: &str = "^(?i)ping(?-i) (.+)$";
const GET_REGEX: &str = "^(?i)get(?-i) ([a-zA-Z0-9]+)$";
const SET_REGEX: &str = "^(?i)set(?-i) ([a-zA-Z0-9]+) (.+)$";
const SUBSCRIBE_REGEX: &str = "^(?i)subscribe(?-i) ([a-zA-Z0-9]+)$";

#[derive(Debug)]
pub enum NonSubscriptionCmdType {
    Exit,
    Ping,
    PingValue(Vec<u8>),
    Set(String, Vec<u8>),
    Get(String),
    Subscribe(String),
    Other,
}

#[derive(Debug)]
pub enum SubscriptionCmdType {
    Publish(Vec<u8>),
    Unsubscribe,
}

pub fn parse_non_subscription_command(command: Vec<u8>) -> NonSubscriptionCmdType {
    let command_str = String::from_utf8_lossy(&command);
    let command_str = command_str.trim();
    if is_exit(command_str) {
        NonSubscriptionCmdType::Exit
    } else if is_ping(command_str) {
        NonSubscriptionCmdType::Ping
    } else if is_ping_value(command_str) {
        let value = extract_ping_with_value(command_str).unwrap_or(Vec::new());
        NonSubscriptionCmdType::PingValue(value)
    } else if is_get(command_str) {
        let key = extract_get(command_str);
        NonSubscriptionCmdType::Get(key.to_owned())
    } else if is_set(command_str) {
        let (key, value) = extract_set(command_str);
        NonSubscriptionCmdType::Set(key.to_owned(), value.to_owned())
    } else if is_subscribe(command_str) {
        let topic = extract_subscribe(command_str);
        NonSubscriptionCmdType::Subscribe(topic.to_owned())
    } else {
        NonSubscriptionCmdType::Other
    }
}

pub fn parse_subscription_command(command: Vec<u8>) -> SubscriptionCmdType {
    let command_str = String::from_utf8_lossy(&command);
    let command_str = command_str.trim();
    if is_unsubscribe(command_str) {
        SubscriptionCmdType::Unsubscribe
    } else {
        SubscriptionCmdType::Publish(command)
    }
}

fn is_exit(command: &str) -> bool {
    command == "exit"
}

fn is_ping(command: &str) -> bool {
    command == "ping"
}

fn is_ping_value(command: &str) -> bool {
    Regex::new(PING_VALUE_REGEX)
        .unwrap()
        .captures(command)
        .is_some()
}

fn extract_ping_with_value(command: &str) -> Option<Vec<u8>> {
    Regex::new(PING_VALUE_REGEX)
        .unwrap()
        .captures(command)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_owned().into_bytes())
}

fn is_get(command: &str) -> bool {
    Regex::new(GET_REGEX).unwrap().captures(command).is_some()
}

fn extract_get(command: &str) -> &str {
    Regex::new(GET_REGEX)
        .unwrap()
        .captures(command)
        .map(|c| {
            let (_, [key]) = c.extract();
            key
        })
        .unwrap()
}

fn is_set(command: &str) -> bool {
    Regex::new(SET_REGEX).unwrap().captures(command).is_some()
}

fn extract_set(command: &str) -> (&str, Vec<u8>) {
    Regex::new(SET_REGEX)
        .unwrap()
        .captures(command)
        .map(|c| {
            let (_, [key, value]) = c.extract();
            (key, value.as_bytes().to_vec())
        })
        .unwrap()
}

fn is_subscribe(command: &str) -> bool {
    Regex::new(SUBSCRIBE_REGEX)
        .unwrap()
        .captures(command)
        .is_some()
}

fn extract_subscribe(command: &str) -> &str {
    Regex::new(SUBSCRIBE_REGEX)
        .unwrap()
        .captures(command)
        .map(|c| {
            let (_, [topic]) = c.extract();
            topic
        })
        .unwrap()
}

fn is_unsubscribe(command: &str) -> bool {
    command == "unsubscribe"
}
