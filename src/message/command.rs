use std::fmt::{self, Display};

#[derive(Debug, PartialEq)]
pub enum Command {
    Numeric(u32),
    Cap,
    Notice,
    Nick,
    User,
    Join,
    PrivMsg,
    Ping,
    Pong,
    Error,
    RplWelcome,
    RplYourHost,
    RplCreated,
    RplMyInfo,
    MessageOfTheDay,
    RplMotd,
    RplMotdStart,
    RplEndOfMotd,
    Unknown(String),
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s {
            "NOTICE" => Command::Notice,
            "PRIVMSG" => Command::PrivMsg,
            "NICK" => Command::Nick,
            "CAP" => Command::Cap,
            "USER" => Command::User,
            "PING" => Command::Ping,
            "PRIVMSG" => Command::PrivMsg,
            "ERROR" => Command::Error,
            "MOTD" => Command::MessageOfTheDay,
            "RPL_MOTD" => Command::RplMotd,
            "RPL_MOTDSTART" => Command::RplMotdStart,
            "RPL_ENDOFMOTD" => Command::RplEndOfMotd,
            _ => match s {
                "001" => Command::RplWelcome,
                "002" => Command::RplYourHost,
                "003" => Command::RplCreated,
                "004" => Command::RplMyInfo,
                "372" => Command::RplMotd,
                "375" => Command::RplMotdStart,
                "376" => Command::RplEndOfMotd,
                _ => {
                    Command::Unknown(s.to_string())
                }
            },
        }
    }
}

impl PartialEq<String> for Command {
    fn eq(&self, rhs: &String) -> bool {
        &self.to_string() == rhs
    }
}

impl PartialEq<&Command> for String { 
    fn eq(&self, rhs: &&Command) -> bool {
        self == &rhs.to_string()
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = match self {
            Command::Numeric(n) => {
                format!("{}", n)
            }
            Command::Unknown(s) => s.clone(),
            Command::Cap => "CAP".to_string(),
            Command::Notice => "NOTICE".to_string(),
            Command::Nick => "NICK".to_string(),
            Command::User => "USER".to_string(),
            Command::PrivMsg => "PRIVMSG".to_string(),
            Command::Ping => "PING".to_string(),
            Command::Pong => "PONG".to_string(),
            Command::Join => "JOIN".to_string(),
            Command::RplWelcome => "RPL_WELCOME".to_string(),
            Command::MessageOfTheDay => "MOTD".to_string(),
            Command::Error => "ERROR".to_string(),
            Command::RplMotd => "RPL_MOTD".to_string(),
            Command::RplMotdStart => "RPL_MOTDSTART".to_string(),
            Command::RplEndOfMotd => "RPL_ENDOFMOTD".to_string(),
            Command::RplWelcome => "RPL_WELCOME".to_string(),
            Command::RplYourHost => "RPL_YOURHOST".to_string(),
            Command::RplCreated => "RPL_CREATED".to_string(),
            Command::RplMyInfo => "RPL_MYINFO".to_string()
        };
        write!(f, "{}", command)
    }
}