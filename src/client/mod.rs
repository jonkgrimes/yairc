use crate::message::{Command, Message};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Client {
    sender: Arc<Mutex<Sender<Message>>>,
    receiver: Arc<Mutex<Receiver<Message>>>,
}

impl Client {
    pub fn new(server: &str, channel_name: &str, nick: &str) -> Self {
        let (rx, tx) = channel();
        let sender = Arc::new(Mutex::new(rx));
        let receiver = Arc::new(Mutex::new(tx));

        Self { sender, receiver }
    }

    pub fn sender(&self) -> Arc<Mutex<Sender<Message>>> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Arc<Mutex<Receiver<Message>>> {
        self.receiver.clone()
    }
}

pub fn register(nick: &str) -> Vec<Message> {
    vec![
        Message::new(Command::Cap, vec!["LS", "302"]),
        Message::new(Command::Nick, vec![nick]),
        Message::new(Command::User, vec![&format!("{} 0 * :Developer", nick)]),
        Message::new(Command::Cap, vec!["END"]),
    ]
}

pub fn join(channel_name: &str) -> Vec<Message> {
    let channel_name = format!("#{}", channel_name);
    vec![Message::new(Command::Join, vec![&channel_name])]
}
