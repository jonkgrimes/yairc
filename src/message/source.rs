use std::fmt::{self, Display};

#[derive(Debug, PartialEq)]
pub struct Source {
    pub nick: String,
    pub user: Option<String>,
    pub host: Option<String>,
}

impl Source {
    pub fn new(nick: String) -> Source {
        Source {
            nick: nick,
            user: None,
            host: None,
        }
    }

    pub fn new_with_user_and_host(nick: String, user: String, host: String) -> Source {
        Source {
            nick: nick,
            user: Some(user),
            host: Some(host)
        }
    }
}

impl From<String> for Source {
    fn from(s: String) -> Self {
        Self { nick: s, user: None, host: None }
    }
}

impl From<(&str, Option<&str>, Option<&str>)> for Source {
    fn from(s: (&str, Option<&str>, Option<&str>)) -> Self {
        Self {
            nick: s.0.to_string(),
            user: s.1.map(|s| s.to_string()),
            host: s.2.map(|s| s.to_string()),
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.user.is_some() && self.host.is_some() {
            write!(f, "{}!{}@{}", self.nick, self.user.as_ref().unwrap(), self.host.as_ref().unwrap())
        } else {
            write!(f, "{}", self.nick)
        }
    }
}

impl PartialEq<Source> for String { 
    fn eq(&self, rhs: &Source) -> bool {
        self == &rhs.nick
    }
}
