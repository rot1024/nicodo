use chrono::{Duration as RawDuration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum DateTime {
    Posted,
    PostedPlus(Duration),
    Latest,
    DateTime(NaiveDateTime),
}

impl DateTime {
    pub fn datetime(&self, posted_date_time: NaiveDateTime) -> NaiveDateTime {
        match self {
            Self::Posted => posted_date_time,
            Self::PostedPlus(d) => posted_date_time + d.duration(),
            Self::Latest => Local::now().naive_local(),
            Self::DateTime(d) => *d,
        }
    }
}

impl FromStr for DateTime {
    type Err = &'static str;

    fn from_str(dt: &str) -> Result<DateTime, &'static str> {
        const FORMAT1: &str = "%Y-%m-%d %H:%M:%S";
        const FORMAT2: &str = "%Y-%m-%d";
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^posted\+(.+)$").unwrap();
        }

        if dt == "latest" {
            return Ok(Self::Latest);
        } else if dt == "posted" {
            return Ok(Self::Posted);
        }
        if let Some(c) = RE.captures_iter(dt).next() {
            return Ok(Self::PostedPlus(c[1].parse::<Duration>()?));
        }
        let dt = NaiveDateTime::parse_from_str(dt, FORMAT1)
            .or_else(|_| {
                NaiveDate::parse_from_str(dt, FORMAT2)
                    .map(|d| NaiveDateTime::new(d, NaiveTime::from_hms(0, 0, 0)))
            })
            .map_err(|_| "invalid datetime")?;

        Ok(Self::DateTime(dt))
    }
}

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(RawDuration, String);

impl Duration {
    pub fn duration(&self) -> RawDuration {
        self.0
    }

    pub fn source(&self) -> &str {
        &self.1
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl FromStr for Duration {
    type Err = &'static str;

    fn from_str(d: &str) -> Result<Duration, &'static str> {
        const ERR: &str = "invalid duration";
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)([hdw])$").unwrap();
        }
        match RE.captures_iter(d).next() {
            Some(c) => {
                let val = c[1].parse::<u64>().map_err(|_| ERR)?;
                match &c[2] {
                    "h" => Ok(Duration(RawDuration::hours(val as i64), d.into())),
                    "d" => Ok(Duration(RawDuration::days(val as i64), d.into())),
                    "w" => Ok(Duration(RawDuration::days(val as i64 * 7), d.into())),
                    _ => Err(ERR),
                }
            }
            None => Err(ERR),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_datetime() {
        let dt = NaiveDateTime::from_str("2019-01-01T00:00:00").unwrap();
        let d = "posted".parse::<DateTime>().unwrap();
        assert_eq!(
            d.datetime(dt),
            NaiveDateTime::from_str("2019-01-01T00:00:00").unwrap()
        );
        let d = "posted+1w".parse::<DateTime>().unwrap();
        assert_eq!(
            d.datetime(dt),
            NaiveDateTime::from_str("2019-01-08T00:00:00").unwrap()
        );
        let d = "2010-01-01 10:01:02".parse::<DateTime>().unwrap();
        assert_eq!(
            d.datetime(dt),
            NaiveDateTime::from_str("2010-01-01T10:01:02").unwrap()
        );
        let d = "2010-01-01".parse::<DateTime>().unwrap();
        assert_eq!(
            d.datetime(dt),
            NaiveDateTime::from_str("2010-01-01T00:00:00").unwrap()
        );
    }

    #[test]
    fn test_duration() {
        let d = "1h".parse::<Duration>().unwrap();
        assert_eq!(d.duration(), RawDuration::hours(1));
        let d = "14h".parse::<Duration>().unwrap();
        assert_eq!(d.duration(), RawDuration::hours(14));
        let d = "1d".parse::<Duration>().unwrap();
        assert_eq!(d.duration(), RawDuration::days(1));
        let d = "10d".parse::<Duration>().unwrap();
        assert_eq!(d.duration(), RawDuration::days(10));
        let d = "1w".parse::<Duration>().unwrap();
        assert_eq!(d.duration(), RawDuration::days(7));
        let d = "7w".parse::<Duration>().unwrap();
        assert_eq!(d.duration(), RawDuration::days(49));
    }
}
