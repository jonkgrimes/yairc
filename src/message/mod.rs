use crate::message::parser::message;

mod parser;

#[derive(Debug, PartialEq)]
struct Tag(String, String);
type Tags = Vec<Tag>;

#[derive(Debug, PartialEq)]
struct Source(String);

#[derive(Debug, PartialEq)]
pub enum Command {
    Numeric(u32),
    Notice,
    PrivMsg,
    Ping,
    Pong,
    Unknown
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s {
            "NOTICE" => Command::Notice,
            "PRIVMSG" => Command::PrivMsg,
            _ => Command::Unknown
        }
    }
}

#[derive(Debug, PartialEq)]
struct Param(String);
type Params = Vec<Param>;

#[derive(Debug, PartialEq)]
pub struct Message {
    tags: Option<Tags>,
    source: Option<Source>,
    command: Command,
    params: Params
}

impl Message {
    pub fn new(command: Command, params: Params) -> Self {
        Message { tags: None, source: None, command, params }
    }

    pub fn ping() -> Self {
        Message { tags: None, source: None, command: Command::Ping, params: vec![] }
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
                let tags = tags.iter().map(|t| Tag(t.0.to_string(), t.1.to_string())).collect();
                Some(tags)
            }
            None => None
        };
        let source = match source {
            None => None,
            Some(source) => Some(Source(source.to_string()))
        };
        let command = Command::from(command);
        let params = params.iter().map(|p| Param(p.to_string())).collect();
        Ok(Self {
            tags,
            source,
            command,
            params
        })
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
            tags: Some(vec![Tag("id".to_string(), "123".to_string()), Tag("type".to_string(), "something".to_string())]),
            source: Some(Source("Guest1".to_string())), // source
            command: Command::PrivMsg,
            params: vec![Param("#test_123".to_string()), Param("Hello".to_string())],                                // paramerters
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
            params: vec![Param("*".to_string()), Param("*** Looking up your hostname...".to_string())],                                // paramerters
        };
        assert_eq!(actual, expected);
    }
}