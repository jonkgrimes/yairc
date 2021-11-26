use std::net::TcpStream;
use crate::message::{Message, Command};

pub struct Client;

impl Client {
    pub fn join_channel(conn: &mut TcpStream, name: String) {
        
    }
}

pub fn register() -> Vec<Message> {
    vec![
        Message::new(Command::Cap, vec!["LS", "302"]),
        Message::new(Command::Nick, vec!["Dev123"]),
        Message::new(Command::User, vec!["Dev123 0 * :Developer"]),
        Message::new(Command::Cap, vec!["END"])
    ]
}

pub fn join(channel_name: &str) -> Vec<Message> {
    vec![
        Message::new(Command::Join, vec![channel_name]),
    ]
}