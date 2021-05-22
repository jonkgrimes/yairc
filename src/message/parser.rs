//! IRC Parser written with nom

use std::str;

use nom::{IResult, bytes::complete::take_until, character::is_alphanumeric, combinator::{cond, opt, peek}, sequence::terminated};
use nom::dbg_basic;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alphanumeric0, space1, crlf};
use nom::sequence::separated_pair;
use nom::multi::separated_list0;

// Basic message structure
// [@tags] [:source] <command> <parameters>

// Tag parsers
fn tag_start(i: &str) -> IResult<&str, &str> {
    tag("@")(i)
}

fn tag_separator(i: &str) -> IResult<&str, &str> {
    tag(";")(i)
}

fn tag_key(i: &str) -> IResult<&str, &str> {
    let f = |c: char| { is_alphanumeric(c as u8) || c == '-' };
    take_while(f)(i)
}

fn tag_end(i: &str) -> IResult<&str, &str> {
    alt((tag(";"), tag(" ")))(i)
}

fn tag_pair(i: &str) -> IResult<&str, (&str, &str)> {
   separated_pair(alphanumeric0, tag("="), alphanumeric0)(i)
}

fn tags(i: &str) -> IResult<&str, Option<Vec<(&str, &str)>>> {
    let (i, o) = tag_start(i)?;
    if o == "@" {
        let (rest, tags) = separated_list0(tag_separator, tag_pair)(i)?;
        if tags.len() == 0 {
            Ok((rest, None))
        } else {
            Ok((rest, Some(tags)))
        }
    } else {
        Ok((i, None))
    }
}

// Source parsers
fn source_start(i: &str) -> IResult<&str, &str> {
    tag(":")(i)
}

fn source(i: &str) -> IResult<&str, &str> {
    take_until("!")(i)
}

fn client(i: &str) -> IResult<&str, &str> {
    take_until("@")(i)
}

// Command parsers
fn command(i: &str) -> IResult<&str, &str> {
    take_until(" ")(i)
}

// Parameter parsers
fn message_part(i: &str) -> IResult<&str, &str> {
    take_until(" ")(i)
}

fn params(i: &str) -> IResult<&str, &str> {
    take_until("\r\n")(i)
}

fn message(i: &str) -> IResult<&str, Vec<&str>> {
    println!("Executing...");
    // let mut result: Vec<&str> = Vec::new();
    // if tags present parse
    let (i, tags) = tags(i)?;
    println!("{}", i);
    if let Some(tags) = tags {
        dbg!(tags);
    }
    let (i, source) = source(i)?;
    if source != "" {
        dbg!(source);
    }
    let (i, command) = command(i)?;
    dbg!(command);
    // let (i) = peek(tag_start)?;
    // if source present, parse
    // parse command
    // parse params
    let(i, params) = params(i)?;
    dbg!(params);

    Ok(("", vec!["", source, command, params]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tag_key() {
        let raw = "some-key-123";
        let (_ , actual) = tag_key(raw).unwrap();
        let expected = "some-key-123";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tag_key_error() {
        let raw = "some-key#";
        let (_ , actual) = tag_key(raw).unwrap();
        let expected = "some-key";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_line_start() {
        let raw = ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let (_, start) = source_start(raw).unwrap();
        let expected = ":";
        let actual= start;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_source() {
        let raw = ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let (i, _) = source_start(raw).unwrap();
        let (_i, source) = source(i).unwrap();
        let expected = "Guest1";
        let actual = source;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tag() {
        let raw = "id=123AB";
        let (i, actual) = tag_pair(raw).unwrap();
        let expected = ("id", "123AB");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tag_with_number_value() {
        let raw = "id=123";
        let (i, actual) = tag_pair(raw).unwrap();
        let expected = ("id", "123");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_no_value_tag() {
        let raw = "type=";
        let (i, actual) = tag_pair(raw).unwrap();
        let expected = ("type", "");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags() {
        let raw = "id=123;type=sometype ";
        let (i, actual) = tags(raw).unwrap();
        let expected = vec![("id", "123"),("type","sometype")];
        // assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags_with_true_terminator() {
        let raw = "id=123;type= ";
        let (i, actual) = tags(raw).unwrap();
        let expected = vec![("id", "123"),("type","")];
        // assert_eq!(actual, expected);
    }

    #[test]
    fn test_top_level_parser() {
        let raw = "@id=123;type=something :Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let (i, actual) = message(raw).unwrap();
        let expected = vec![
            "@id=123;type=something ",                        // tags
            ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP ", // source
            "PRIVMSG ",                                       // command
            "#test_123 :Hello"                                // paramerters
        ];
        assert_eq!(actual, expected);
    }
}