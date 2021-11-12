use std::fmt;
use std::fmt::Display;
use std::iter::FromIterator;

mod parser;
mod source;

use crate::message::parser::message;
use crate::message::source::Source;

#[derive(Debug, PartialEq)]
struct Tag(String, String);
type Tags = Vec<Tag>;

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
            _ => match s {
                "001" => Command::RplWelcome,
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
            Command::Error => "ERROR".to_string(),
        };
        write!(f, "{}", command)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param(String);

impl Param {
    pub fn new(param: &str) -> Self {
        Param(param.to_string())
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Param {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Param {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl PartialEq<String>  for Param {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

#[derive(Debug, PartialEq)]
pub struct Params {
    params: Vec<Param>,
}


impl Params {
    fn new() -> Self {
        Self { params: Vec::new() }
    }

    fn add(&mut self, param: Param) {
        self.params.push(param)
    }

    pub fn get(&self, index: usize) -> Option<&Param> {
        self.params.get(index)
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.params.iter().map(|p| p.0.clone()).collect()
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self
            .params
            .iter()
            .map(|p| format!("{}", p.0))
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", s)
    }
}

impl From<Vec<&str>> for Params {
    fn from(v: Vec<&str>) -> Self {
        Self {
            params: v.iter().map(|s| Param::new(s)).collect(),
        }
    }
}

impl From<[Param; 1]> for Params {
    fn from(a: [Param; 1]) -> Self {
        Self {
            params: a.iter().map(|p| p.clone()).collect(),
        }
    }
}

impl FromIterator<Param> for Params {
    fn from_iter<I: IntoIterator<Item = Param>>(iter: I) -> Self {
        let mut c = Params::new();

        for i in iter {
            c.add(i)
        }

        c
    }
}

#[derive(Debug, PartialEq)]
pub struct Message {
    tags: Option<Tags>,
    source: Option<Source>,
    command: Command,
    params: Option<Params>
}

impl PartialEq<Vec<String>> for Message {
    fn eq(&self, other: &Vec<String>) -> bool {
        match &self.params {
          Some(p) => &p.params == other,
          None => false
        }
    }
}

impl Message {
    pub fn new(command: Command, params: Vec<&str>) -> Self {
        Message {
            tags: None,
            source: None,
            command,
            params: Some(Params::from(params)),
        }
    }

    pub fn ping() -> Self {
        Message {
            tags: None,
            source: None,
            command: Command::Ping,
            params: None
        }
    }

    pub fn pong(server: Param) -> Self {
        Message {
            tags: None,
            source: None,
            command: Command::Pong,
            params: Some(Params::from([server]))
        }
    }

    /// Get a reference to the message's tags.
    pub fn tags(&self) -> Option<&Tags> {
        self.tags.as_ref()
    }

    /// Get a reference to the message's source.
    pub fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn parse(raw: &str) -> Result<Self, Box<dyn std::error::Error + '_>> {
        let (i, (tags, source, command, params)) = message(raw)?;
        let tags = match tags {
            Some(tags) => {
                let tags = tags
                    .iter()
                    .map(|t| Tag(t.0.to_string(), t.1.to_string()))
                    .collect();
                Some(tags)
            }
            None => None,
        };
        let source = match source {
            None => None,
            Some(source) => Some(Source::from(source)),
        };
        let command = Command::from(command);
        let params = params.map(|p| p.iter().map(|p| Param::from(*p)).collect());
        Ok(Self {
            tags,
            source,
            command,
            params,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let s = if let Some(params) = &self.params {
            format!("{} {}\r\n", self.command, params)
        } else {
            format!("{}\r\n", self.command)
        };
        s.into_bytes()
    }

    pub fn get_param(&self, index: usize) -> Option<&Param> {
        match &self.params {
            Some(p) => {
                p.get(index)
            }
            _ => None
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        s.push_str(&format!("{}", self.command));
        if let Some(params) = &self.params {
            s.push_str(&format!(" {}", params));
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let msg = Message::new(Command::Cap, vec!["LS", "302"]);
    }

    #[test]
    fn test_parse() {
        let raw = "@id=123;type=something :Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let actual = Message::parse(raw).unwrap();
        let expected = Message {
            tags: Some(vec![
                Tag("id".to_string(), "123".to_string()),
                Tag("type".to_string(), "something".to_string()),
            ]),
            source: Some(Source::new_with_user_and_host("Guest1".to_string(), "textual".to_string(), "254D99FE.73C022D0.AC18634F.IP".to_string())), // source
            command: Command::PrivMsg,
            params: Some(Params::from(vec!["#test_123", "Hello"])), // paramerters
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_no_tags() {
        let raw = ":irc.jonkgrimes.com NOTICE * :*** Looking up your hostname...\r\n";
        let actual = Message::parse(raw).unwrap();
        let expected = Message {
            tags: None,
            source: Some(Source::new("irc.jonkgrimes.com".to_string())), // source
            command: Command::Notice,
            params: Some(Params::from(vec!["*", "*** Looking up your hostname..."])), // paramerters
        };
        assert_eq!(actual, expected);
    }

    use serde::{Deserialize, Serialize};
    use std::{fs::File, io::Read};
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Deserialize)]
    struct SplitTests {
        tests: Vec<TestCase>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    struct TestCase {
        input: String,
        atoms: Atoms,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Atoms {
        verb: String,
        tags: Option<HashMap<String, String>>,
        source: Option<String>,
        params: Option<Vec<String>>,
    }

    #[test]
    fn parser_integration_tests() {
        let mut yaml =
            File::open("src/message/test_data/msg-split.yaml").expect("Unable to open msg-split.yaml");
        let mut buffer = Vec::new();
        yaml.read_to_end(&mut buffer)
            .expect("Unable to read from file");
        let tests: SplitTests =
            serde_yaml::from_slice(&buffer).expect("Was not in the correct format");
        tests.tests.iter().for_each(|test| {
            let raw = format!("{}\r\n", test.input);
            let message = Message::parse(&raw).expect("Unable to parse message");

            if let Some(params) = &test.atoms.params {
                let msg_params = message.params.as_ref().map(|p| p.to_vec());
                assert_eq!(msg_params.as_ref(), Some(params));
            }

            assert_eq!(test.atoms.verb, message.command());

            if let Some(source) = &test.atoms.source {
                assert_eq!(source, message.source().unwrap());
            }
        });
    }
}
