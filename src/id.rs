use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum Id {
    Video(String),
    Channel(String),
}

impl FromStr for Id {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new("https://ch.nicovideo.jp/(.+)").unwrap();
        }

        if let Some(c) = RE.captures(s).and_then(|c| c.get(1)) {
            return Ok(Self::Channel(c.as_str().to_string()));
        }

        return Ok(Self::Video(
            s.replace("https://www.nicovideo.jp/watch/", ""),
        ));
    }
}

#[test]
fn test_id_parse() {
    assert_eq!("xxx".parse::<Id>().unwrap(), Id::Video("xxx".to_string()));
    assert_eq!(
        "https://www.nicovideo.jp/watch/yyy".parse::<Id>().unwrap(),
        Id::Video("yyy".to_string())
    );
    assert_eq!(
        "https://ch.nicovideo.jp/zzz".parse::<Id>().unwrap(),
        Id::Channel("zzz".to_string())
    );
}
