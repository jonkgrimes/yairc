use std::net::TcpStream;
use crate::message::{Message, Command};

pub struct Client;

impl Client {
    pub fn join_channel(conn: &mut TcpStream, name: String) {
        
    }
}

pub fn register(nick: &str) -> Vec<Message> {
    vec![
        Message::new(Command::Cap, vec!["LS", "302"]),
        Message::new(Command::Nick, vec![nick]),
        Message::new(Command::User, vec![&format!("{} 0 * :Developer", nick)]),
        Message::new(Command::Cap, vec!["END"])
    ]
}

pub fn join(channel_name: &str) -> Vec<Message> {
    let channel_name = format!("#{}", channel_name);
    vec![
        Message::new(Command::Join, vec![&channel_name]),
    ]
}