use crate::message::parser::message;
use std::fmt;
use std::fmt::Display;
use std::iter::FromIterator;

mod parser;

#[derive(Debug, PartialEq)]
struct Tag(String, String);
type Tags = Vec<Tag>;

#[derive(Debug, PartialEq)]
struct Source(String);

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
    Unknown,
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
            _ => {
                match s {
                    "001" => Command::RplWelcome,
                    _ => {
                        eprintln!("Unknown command: {}", s);
                        Command::Unknown
                    }
                }
            }
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = match self {
            Command::Numeric(n) => {
               format!("{}", n)
            },
            Command::Cap => "CAP".to_string(),
            Command::Notice => "NOTICE".to_string(),
            Command::Nick => "NICK".to_string(),
            Command::User => "USER".to_string(),
            Command::PrivMsg => "PRIVMSG".to_string(),
            Command::Ping => "PING".to_string(),
            Command::Pong => "PONG".to_string(),
            Command::Unknown => "UNKNOWN".to_string(),
            Command::Join => "JOIN".to_string(),
            Command::RplWelcome => "RPL_WELCOME".to_string(),
            Command::Error => "ERROR".to_string()
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

impl From<&str> for Param {
    fn from(s: &str) -> Self {
        Self(s.to_string())
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
            params: a.iter().map(|p| p.clone()).collect()
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
    params: Params,
}

impl Message {
    pub fn new(command: Command, params: Vec<&str>) -> Self {
        Message {
            tags: None,
            source: None,
            command,
            params: Params::from(params),
        }
    }

    pub fn ping() -> Self {
        Message {
            tags: None,
            source: None,
            command: Command::Ping,
            params: Params::new()
        }
    }

    pub fn pong(server: Param) -> Self {
        Message {
            tags: None,
            source: None,
            command: Command::Pong,
            params: Params::from([server])
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
            Some(source) => Some(Source(source.to_string())),
        };
        let command = Command::from(command);
        let params = params.iter().map(|p| Param::from(*p)).collect();
        Ok(Self {
            tags,
            source,
            command,
            params,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let s= format!("{} {}\r\n", self.command, self.params);
        s.into_bytes()
    }

    pub fn get_param(&self, index: usize) -> Option<&Param> {
        self.params.get(index)
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
            source: Some(Source("Guest1".to_string())), // source
            command: Command::PrivMsg,
            params: Params::from(vec!["#test_123", "Hello"]) // paramerters
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_no_tags() {
        let raw = ":irc.jonkgrimes.com NOTICE * :*** Looking up your hostname...\r\n";
        let actual = Message::parse(raw).unwrap();
        let expected = Message {
            tags: None,
            source: Some(Source("irc.jonkgrimes.com".to_string())), // source
            command: Command::Notice,
            params: Params::from(vec![
                "*",
                "*** Looking up your hostname..."
            ]), // paramerters
        };
        assert_eq!(actual, expected);
    }
}
