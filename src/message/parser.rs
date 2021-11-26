//! IRC Parser written with nom

use std::str;

use nom::bytes::complete::{escaped, tag, take, take_while, take_while_m_n};
use nom::sequence::{preceded, delimited};
use nom::character::complete::{alphanumeric0, crlf, multispace0, space0, char};
use nom::combinator::{recognize, value};
use nom::multi::{separated_list0};
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
    Ok((i, unescaped_value))
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

fn user(i: &str) -> IResult<&str, &str> {
    let f = |c: char| is_alphanumeric(c as u8) || c == '-' || c == '.' || c == '_';
    take_while(f)(i)
}

fn host(i: &str) -> IResult<&str, &str> {
    let f = |c: char| is_alphanumeric(c as u8) || c == '-' || c == '.' || c == '_';

    // let (i, host) = escaped(alphanumeric0, '\\', one_of(r#"u\{\}"#))(i)?;
    take_while(f)(i)
}

fn user_and_host(i: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(user, tag("@"), host)(i)
}

fn nick_user_and_host(i: &str) -> IResult<&str, (&str, (&str, &str))> {
    separated_pair(alphanumeric0, tag("!"), user_and_host)(i)
}

fn source(i: &str) -> IResult<&str, Option<(&str, Option<&str>, Option<&str>)>> {
    // No source
    let (i, o) = source_start(i)?;
    if o == None {
        return Ok((i, None));
    }

    let (i, source) = terminated(take_until(" "), space0)(i)?;
    // ":irc.jonkgrimes.com" or ":Guest24!user@localhost"
    match recognize(nick_user_and_host)(source) {
        Ok((_, source)) => {
            let (_, (nick, (user, host))) = nick_user_and_host(source)?;
            Ok((i, Some((nick, Some(user), Some(host)))))
        }
        Err(_) => Ok((i, Some((source, None, None))))
    }
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
    let (i, _) = multispace0(i)?;
    let (i, (params, _)) = many_till(param, crlf)(i)?;
    Ok((i, params))
}

fn param(i: &str) -> IResult<&str, &str> {
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
) -> IResult<&str, (Option<Vec<(&str, &str)>>, Option<(&str, Option<&str>, Option<&str>)>, &str, Option<Vec<&str>>)> {
    let (i, tags) = tags(i)?;
    let (i, source) = source(i)?;
    let (i, command) = command(i)?;
    let (i, params) = opt(params)(i)?;

    Ok((i, (tags, source, command, params)))
}

fn unicode_control_character(i: &str) -> IResult<&str, &str> {
  // let parse_hex = take_while_m_n(0, 6, |c: char| c.is_ascii_hexdigit());
  dbg!(i);
  Ok(("", ""))
}

fn control_charater(i: &str) -> IResult<&str, &str> {
    dbg!(i);
    preceded(
        char('\\'),
        // `delimited` is like `preceded`, but it parses both a prefix and a suffix.
        // It returns the result of the middle parser. In this case, it parses
        // {XXXX}, where XXXX is 1 to 6 hex numerals, and returns XXXX
        alt((
          unicode_control_character,
          value("\u{3}", tag("u{3}"))
        ))
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escaped_control_charater() {
        let raw = "\u{3}";
        let actual = control_charater(raw);
        let expected = ("", "\u{3}");
        assert_eq!(actual, Ok(expected))
    }

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
        let expected = Some(("irc.jonkgrimes.com", None, None));
        let actual = source;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_source_with_client() {
        let raw = ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP ";
        let (_i, source) = source(raw).unwrap();
        let expected = Some(("Guest1", Some("textual"), Some("254D99FE.73C022D0.AC18634F.IP")));
        let actual = source;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_host_with_control_characters() {
        let raw = ":Guest1!tex\x30tual@localhost ";
        let (_i, source) = source(raw).unwrap();
        let expected = Some(("Guest1", Some("tex\x30tual"), Some("localhost ")));
        let actual = source;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_host_with_unicode_characters() {
        let raw = ":Guest1!tex\u{3}tual@localhost ";
        let (_i, source) = source(raw).unwrap();
        let expected = Some(("Guest1", Some("tex\u{3}tual"), Some("localhost")));
        let actual = source;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tag() {
        let raw = "id=123AB";
        let (_, actual) = tag_pair(raw).unwrap();
        let expected = ("id", "123AB");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tag_with_number_value() {
        let raw = "id=123";
        let (_, actual) = tag_pair(raw).unwrap();
        let expected = ("id", "123");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_no_value_tag() {
        let raw = "type=";
        let (_, actual) = tag_pair(raw).unwrap();
        let expected = ("type", "");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags() {
        let raw = "@id=123;type=sometype ";
        let (_, actual) = tags(raw).unwrap();
        let expected = Some(vec![("id", "123"), ("type", "sometype")]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags_with_true_terminator() {
        let raw = "@id=123;type= ";
        let (_,  actual) = tags(raw).unwrap();
        let expected = vec![("id", "123"), ("type", "")];
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_command_parsing() {
        let raw = "NOTICE * :*** Looking up your hostname...";
        let (_, actual) = command(raw).unwrap();
        let expected = "NOTICE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_trailing_param() {
        let raw = "*** Looking up your hostname...\r\n";
        let (_, actual) = trailing_param(raw).unwrap();
        let expected = "*** Looking up your hostname...";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_params_parsing() {
        let raw = "* :*** Looking up your hostname...\r\n";
        let (_, actual) = params(raw).unwrap();
        let expected = vec!["*", "*** Looking up your hostname..."];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_preceding_whitespace_params() {
        let raw = " * :*** Looking up your hostname...\r\n";
        let (_, actual) = params(raw).unwrap();
        let expected = vec!["*", "*** Looking up your hostname..."];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_message_parsing() {
        let raw = ":irc.jonkgrimes.com NOTICE * :*** Looking up your hostname...\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some(("irc.jonkgrimes.com", None, None)),
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
            Some(("irc.example.com", None, None)),
            "CAP",
            Some(vec!["*", "LIST", ""]),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_space_preceding_params() {
        let raw = ":irc.example.com CAP  * LIST :\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some(("irc.example.com", None, None)),
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
            Some(("dan", Some("d"), Some("localhost"))),
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
            Some(("dan", Some("d"), Some("localhost"))),
            "PRIVMSG",
            Some(vec!["#chan", "Hey!"])
        );
        assert_eq!(actual, expected);
    }
}
