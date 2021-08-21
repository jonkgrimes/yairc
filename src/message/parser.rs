//! IRC Parser written with nom

use std::str;

use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::complete::{alphanumeric0, line_ending};
use nom::multi::{separated_list0};
use nom::sequence::{preceded, separated_pair};
use nom::{
    bytes::complete::take_until, character::is_alphanumeric, combinator::opt, multi::many_till,
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
    let f = |c: char| is_alphanumeric(c as u8) || c == '-';
    take_while(f)(i)
}

fn tag_pair(i: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(tag_key, tag("="), alphanumeric0)(i)
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
    let (i, o) = tag("!")(i)?;
    terminated(take_until(" "), tag(" "))(i)
}

// Command parsers
fn command(i: &str) -> IResult<&str, &str> {
    let (i, command) = take_until(" ")(i)?;
    let (i, _) = tag(" ")(i)?;
    Ok((i, command))
}

// Parameter parsers
fn params(i: &str) -> IResult<&str, Vec<&str>> {
    let (i, (mut params, rest)) = many_till(param, tag(":"))(i)?;
    let (i, p) = take_until(LINE_ENDING)(i)?;
    params.push(p);
    Ok((i, params))
}

fn param(i: &str) -> IResult<&str, &str> {
    terminated(take_until(" "), tag(" "))(i)
}

fn trailing_param(i: &str) -> IResult<&str, &str> {
    preceded(tag(":"), terminated(take_until(LINE_ENDING), line_ending))(i)
}

pub fn message(
    i: &str,
) -> IResult<&str, (Option<Vec<(&str, &str)>>, Option<&str>, &str, Vec<&str>)> {
    let (i, tags) = tags(i)?;
    let (i, source) = source(i)?;
    let (i, command) = command(i)?;
    let (i, params) = params(i)?;

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
        let raw = ":*** Looking up your hostname...\r\n";
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
    fn test_message_parsing() {
        let raw = ":irc.jonkgrimes.com NOTICE * :*** Looking up your hostname...\r\n";
        let (_i, actual) = message(raw).unwrap();
        let expected = (
            None,
            Some("irc.jonkgrimes.com"),
            "NOTICE",
            vec!["*", "*** Looking up your hostname..."],
        );
        assert_eq!(actual, expected);
    }
}
