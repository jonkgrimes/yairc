use std::fmt;
use std::fmt::Display;

mod parser;
mod command;
mod source;
mod param;

use crate::message::parser::message;
pub use crate::message::command::Command;
use crate::message::param::{Param, Params};
use crate::message::source::Source;

#[derive(Debug, PartialEq)]
struct Tag(String, String);
type Tags = Vec<Tag>;


#[derive(Debug, PartialEq)]
pub struct Message {
    tags: Option<Tags>,
    source: Option<Source>,
    command: Command,
    params: Option<Params>
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

    pub fn motd() -> Self {
        Message {
            tags: None,
            source: None,
            command: Command::MessageOfTheDay,
            params: None
        }
    }

    pub fn priv_msg(nick: String, message: String) -> Self {
        let source = Some(Source { nick, user: None, host: None });
        let mut params = vec![message];
        Message { tags: None, source , command: Command::PrivMsg, params: Some(params.into()) }
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
