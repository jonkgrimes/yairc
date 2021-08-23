use crate::message::{Message, Command};

fn register() -> [Message; 4] {}
    [
        Message::new(Command::Cap, vec!["LS", "302"],
        Message::new(Command::Nick, vec![""]
        Message::new(Command::User, vec![""]
        Message::new(Command::Cap, vec!["END"])
    ]
}