use nom::{alt, char, named, separated_list0, separated_pair, tag, take_until, dbg_dmp, dbg_basic};
use nom::character::complete::alphanumeric0;

// Tag parsers
named!(tag_start, tag!("@"));
named!(tag_separator<&str, &str>, tag!(";"));
named!(tag_end<&str, &str>, alt!(tag!(";") | tag!(" ")));
named!(tag_pair<&str, (&str, &str)>, separated_pair!(alphanumeric0, tag!("="), alphanumeric0));
named!(tags<&str, Vec<(&str, &str)>>, dbg_dmp!(separated_list0!(tag!(";"), tag_pair)));

// Source parsers
named!(source_start, tag!(":"));
named!(source, take_until!("!"));
named!(client, take_until!("@"));


// Command parsers

// Parameter parsers

named!(line_ending, tag!("\r\n"));

// named!(parse_message, tuple!((line_start, source, client));

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_line_start() {
        let raw = ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let (_, start) = source_start(raw.as_bytes()).unwrap();
        let expected = ":".to_string();
        let actual= String::from_utf8_lossy(start);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_source() {
        let raw = ":Guest1!textual@254D99FE.73C022D0.AC18634F.IP PRIVMSG #test_123 :Hello\r\n";
        let (i, _) = source_start(raw.as_bytes()).unwrap();
        let (_i, source) = source(i).unwrap();
        let expected = "Guest1";
        let actual = String::from_utf8_lossy(source);
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
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags_with_true_terminator() {
        let raw = "id=123;type= ";
        let (i, actual) = tags(raw).unwrap();
        let expected = vec![("id", "123"),("type","")];
        assert_eq!(actual, expected);
    }
}