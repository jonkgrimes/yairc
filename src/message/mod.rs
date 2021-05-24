use crate::message::parser::message;

mod parser;

#[derive(Debug, PartialEq)]
struct Tag(String, String);
type Tags = Vec<Tag>;

#[derive(Debug, PartialEq)]
struct Source(String);

#[derive(Debug, PartialEq)]
enum Command {
    Numeric(u32),
    PrivMsg
}

#[derive(Debug, PartialEq)]
struct Param(String);
type Params = Vec<Param>;

#[derive(Debug, PartialEq)]
struct Message {
    tags: Option<Tags>,
    source: Option<Source>,
    command: Command,
    params: Params
}

impl Message {
    /// Get a reference to the message's tags.
    fn tags(&self) -> Option<&Tags> {
        self.tags.as_ref()
    }

    /// Get a reference to the message's source.
    fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    fn parse(raw: &str) -> Result<Self, Box<dyn std::error::Error + '_>> {
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
        let command = Command::PrivMsg;
        let params = vec![Param(params.to_string())];
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
    fn test_parse() {
        let raw = "@id=123;type=something :Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let actual = Message::parse(raw).unwrap();
        let expected = Message { 
            tags: Some(vec![Tag("id".to_string(), "123".to_string()), Tag("type".to_string(), "something".to_string())]),
            source: Some(Source(":Guest1!textual@254D99FE.73C022D0.AC18634F.IP ".to_string())), // source
            command: Command::PrivMsg,
            params: vec![Param("#test_123 :Hello".to_string())],                                // paramerters
        };
        assert_eq!(actual, expected);
    }
}