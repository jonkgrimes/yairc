
mod parser;

struct Tag(String);
type Tags = Vec<Tag>;

#[derive(Debug, PartialEq)]
struct Source(String);

enum Command {
    Numeric(u32),
    PrivMsg
}

struct Param(String);
type Params = Vec<Param>;

struct Message {
    tags: Tags,
    source: Source,
    command: Command,
    params: Params
}

impl Message {
    /// Get a reference to the message's tags.
    fn tags(&self) -> &Tags {
        &self.tags
    }

    /// Get a reference to the message's source.
    fn source(&self) -> &Source {
        &self.source
    }

    fn parse(raw: &str) -> Self {
        Self {
            tags: vec![],
            source: Source("Hello".to_owned()),
            command: Command::Numeric(0),
            params: vec![]
        }
    }
}