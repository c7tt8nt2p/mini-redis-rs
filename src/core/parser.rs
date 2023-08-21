use regex::{Captures, Regex};

const PING_REGEX: &str = "^ping ([a-zA-Z0-9]+)$";
pub enum NonSubscriptionCmdType {
    Exit,
    Ping(Vec<u8>),
    Set,
    Get,
    Subscribe,
    Other,
}

pub fn parse_non_subscription_command(command: Vec<u8>) -> NonSubscriptionCmdType {
    let command = String::from_utf8_lossy(&command).trim().to_lowercase();
    let command = command.as_str();
    if is_exit(command) {
        NonSubscriptionCmdType::Exit
    } else if is_ping(command) {
        let value = extract_ping(command).unwrap_or(Vec::new());
        NonSubscriptionCmdType::Ping(value)
    } else if is_set(command) {
        NonSubscriptionCmdType::Set
    } else if is_get(command) {
        NonSubscriptionCmdType::Get
    } else if is_subscribe(command) {
        NonSubscriptionCmdType::Subscribe
    } else {
        NonSubscriptionCmdType::Other
    }
}

fn is_exit(command: &str) -> bool {
    command == "exit"
}

fn is_ping(command: &str) -> bool {
    command.starts_with("ping ")
}

fn extract_ping(command: &str) -> Option<Vec<u8>> {
    Regex::new(PING_REGEX)
        .unwrap()
        .captures(command)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_owned().into_bytes())
}

fn is_get(command: &str) -> bool {
    true
}

fn is_set(command: &str) -> bool {
    true
}

fn is_subscribe(command: &str) -> bool {
    true
}
