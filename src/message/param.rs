use std::fmt::{self, Display};
use std::iter::FromIterator;

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