//! IRC Parser written with nom

use std::str;

use nom::bytes::complete::{escaped, tag, take_till, take_while};
use nom::character::complete::{alphanumeric0, crlf, multispace0, space0, one_of};
use nom::multi::{self, separated_list0};
use nom::branch::alt;
use nom::sequence::{separated_pair};
use nom::{
    bytes::complete::take_until, character::is_alphanumeric, character::is_space, combinator::opt, multi::many_till,
    sequence::terminated, IResult,
};

const LINE_ENDING: &str = "\r\n";

// Basic message structure
// [@tags] [:source] <command> <parameters>

// Tag parsers
fn tag_start(i: &str) -> IResult<&str, Option<&str>> {
    opt(tag("@"))(i)
}

fn tag_separator(i: &str) -> IResult<&str, &str> {
    tag(";")(i)
}

fn tag_key(i: &str) -> IResult<&str, &str> {
    let f = |c: char| is_alphanumeric(c as u8) || c == '-' || c == '/' || c == ' ';
    take_while(f)(i)
}

fn tag_value(i: &str) -> IResult<&str, &str> {
    let f = |c: char| !is_space(c as u8) && c != ';';
    let (i, unescaped_value) = take_while(f)(i)?;
    // let (_, value) = escaped(alphanumeric0, '\\', one_of(r#""n\s"#))(unescaped_value)?;
    Ok((i, unescaped_value))
    // alphanumeric0(i)
}

fn tag_pair(i: &str) -> IResult<&str, (&str, &str)> {
    if let Ok((rest, tag)) = separated_pair(tag_key, tag("="), tag_value)(i) {
        Ok((rest, tag))
    } else {
        // Empty case k1=1;k2;k3=3
        let (rest, tag) = alt((take_until(";"), take_until(" ")))(i)?;
        Ok((rest, (tag, "")))
    }
}

fn tags(i: &str) -> IResult<&str, Option<Vec<(&str, &str)>>> {
    let (i, o) = tag_start(i)?;
    if o == None {
        return Ok((i, None));
    }

    let (rest, tags) = terminated(separated_list0(tag_separator, tag_pair), tag(" "))(i)?;
    if tags.len() == 0 {
        Ok((rest, None))
    } else {
        Ok((rest, Some(tags)))
    }
}

// Source parsers
fn source_start(i: &str) -> IResult<&str, Option<&str>> {
    opt(tag(":"))(i)
}

fn source(i: &str) -> IResult<&str, Option<&str>> {
    // No source
    let (i, o) = source_start(i)?;
    if o == None {
        return Ok((i, None));
    }

    let (i, source) = take_till(|c| c == ' ' || c == '!')(i)?;
    if source.len() > 0 {
        // Ignore the client information for now
        let (i, _) = take_until(" ")(i)?;
        let (i, _) = tag(" ")(i)?;
        Ok((i, Some(source)))
    } else {
        let (i, _) = tag(" ")(i)?;
        Ok((i, None))
    }
}

fn client(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag("!")(i)?;
    terminated(take_until(" "), tag(" "))(i)
}

// Command parsers
fn command(i: &str) -> IResult<&str, &str> {
    let (i, command) = alt(
        (take_until(" "), take_until("\r\n"))
    )(i)?;
    let (i, _) = alt((tag(" "), tag("\r\n")))(i)?;
    Ok((i, command))
}

// Parameter parsers
fn params(i: &str) -> IResult<&str, Vec<&str>> {
    dbg!(i);
    let (i, _) = multispace0(i)?;
    let (i, (params, _)) = many_till(param, crlf)(i)?;
    Ok((i, params))
}

fn param(i: &str) -> IResult<&str, &str> {
    dbg!(i);
    let (i, tag) = opt(tag(":"))(i)?;
    if let Some(_) = tag {
        trailing_param(i)
    } else {
        normal_param(i)
    }
}

fn normal_param(i: &str) -> IResult<&str, &str> {
    let (i, param) = alt((
      terminated(take_until(" "), space0),
      trailing_param
    ))(i)?;
    Ok((i, param))
}

fn trailing_param(i: &str) -> IResult<&str, &str> {
    take_until(LINE_ENDING)(i)
}

pub fn message(
    i: &str,
) -> IResult<&str, (Option<Vec<(&str, &str)>>, Option<&str>, &str, Option<Vec<&str>>)> {
    let (i, tags) = tags(i)?;
    let (i, source) = source(i)?;
    let (i, command) = command(i)?;
    let (i, params) = opt(params)(i)?;

    Ok((i, (tags, source, command, params)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tag_key() {
        let raw = "some-key-123";
        let (_, actual) = tag_key(raw).unwrap();
        let expected = "some-key-123";
        assert_eq!(actual, expected);
    }


    #[test]
    fn test_tag_key_error() {
        let raw = "some-key#";
        let (_, actual) = tag_key(raw).unwrap();
        let expected = "some-key";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tag_empty_value() {
        let raw = "@a=b;c=32;k;rt=ql7 ";
        let (_, actual) = tags(raw).unwrap();
        let expected = vec![("a", "b"), ("c", "32"), ("k", ""), ("rt", "ql7")];
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_source() {
        let raw = ":irc.jonkgrimes.com ";
        let (_i, source) = source(raw).unwrap();
        let expected = Some("irc.jonkgrimes.com");
        let actual = source;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_source_with_client() {
        let raw = ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP ";
        let (_i, source) = source(raw).unwrap();
        let expected = Some("Guest1");
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
        let raw = "@id=123;type=sometype ";
        let (i, actual) = tags(raw).unwrap();
        let expected = Some(vec![("id", "123"), ("type", "sometype")]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags_with_true_terminator() {
        let raw = "@id=123;type= ";
        let (i, actual) = tags(raw).unwrap();
        let expected = vec![("id", "123"), ("type", "")];
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_command_parsing() {
        let raw = "NOTICE * :*** Looking up your hostname...";
        let (i, actual) = command(raw).unwrap();
        let expected = "NOTICE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_trailing_param() {
        let raw = "*** Looking up your hostname...\r\n";
        let (i, actual) = trailing_param(raw).unwrap();
        let expected = "*** Looking up your hostname...";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_params_parsing() {
        let raw = "* :*** Looking up your hostname...\r\n";
        let (i, actual) = params(raw).unwrap();
        let expected = vec!["*", "*** Looking up your hostname..."];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_preceding_whitespace_params() {
        let raw = " * :*** Looking up your hostname...\r\n";
        let (i, actual) = params(raw).unwrap();
        let expected = vec!["*", "*** Looking up your hostname..."];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_message_parsing() {
        let raw = ":irc.jonkgrimes.com NOTICE * :*** Looking up your hostname...\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some("irc.jonkgrimes.com"),
            "NOTICE",
            Some(vec!["*", "*** Looking up your hostname..."]),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_blank_params() {
        let raw = ":irc.example.com CAP * LIST :\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some("irc.example.com"),
            "CAP",
            Some(vec!["*", "LIST", ""]),
        );
        assert_eq!(actual, expected);
    }

    fn test_space_preceding_params() {
        let raw = ":irc.example.com CAP  * LIST :\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some("irc.example.com"),
            "CAP",
            Some(vec!["*", "LIST", ""]),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_long_trailing_param() {
        let raw = "CAP REQ :sasl message-tags foo\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            None,
            "CAP",
            Some(vec!["REQ", "sasl message-tags foo"])
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_privmsg_command() {
        let raw = ":dan!d@localhost PRIVMSG #chan :Hey!\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some("dan"),
            "PRIVMSG",
            Some(vec!["#chan", "Hey!"])
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_no_trailing_param() {
        let raw = ":dan!d@localhost PRIVMSG #chan Hey!\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some("dan"),
            "PRIVMSG",
            Some(vec!["#chan", "Hey!"])
        );
        assert_eq!(actual, expected);
    }
}
