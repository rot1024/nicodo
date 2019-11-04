use super::{Error, Result};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;

pub fn parse_datetime(dt: &str, posted_date_time: NaiveDateTime) -> Result<NaiveDateTime> {
  const FORMAT1: &str = "%Y-%m-%d %H:%M:%S";
  const FORMAT2: &str = "%Y-%m-%d";
  lazy_static! {
    static ref RE: Regex = Regex::new(r"^posted\+(.+)$").unwrap();
  }

  if dt == "posted" {
    return Ok(posted_date_time);
  }
  if let Some(c) = RE.captures_iter(dt).next() {
    return Ok(posted_date_time + parse_duration(&c[1])?);
  }
  if dt == "latest" {
    return Ok(chrono::Local::now().naive_local());
  }
  let dt = NaiveDateTime::parse_from_str(dt, FORMAT1)
    .or_else(|_| {
      NaiveDate::parse_from_str(dt, FORMAT2)
        .map(|d| NaiveDateTime::new(d, NaiveTime::from_hms(0, 0, 0)))
    })
    .map_err(|_| Error::Date)?;
  Ok(dt)
}

pub fn parse_duration(d: &str) -> Result<Duration> {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"^(\d+)([hdw])$").unwrap();
  }
  match RE.captures_iter(d).next() {
    Some(c) => {
      let val = c[1].parse::<u64>().map_err(|_| Error::Duration)?;
      match &c[2] {
        "h" => Ok(Duration::hours(val as i64)),
        "d" => Ok(Duration::days(val as i64)),
        "w" => Ok(Duration::days(val as i64 * 7)),
        _ => Err(Error::Duration),
      }
    }
    None => Err(Error::Duration),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn test_datetime() {
    let dt = NaiveDateTime::from_str("2019-01-01T00:00:00").unwrap();
    let d = parse_datetime("posted", dt).unwrap();
    assert_eq!(d, NaiveDateTime::from_str("2019-01-01T00:00:00").unwrap());
    let d = parse_datetime("posted+1w", dt).unwrap();
    assert_eq!(d, NaiveDateTime::from_str("2019-01-08T00:00:00").unwrap());
    let d = parse_datetime("2010-01-01 10:01:02", dt).unwrap();
    assert_eq!(d, NaiveDateTime::from_str("2010-01-01T10:01:02").unwrap());
    let d = parse_datetime("2010-01-01", dt).unwrap();
    assert_eq!(d, NaiveDateTime::from_str("2010-01-01T00:00:00").unwrap());
  }

  #[test]
  fn test_duration() {
    let d = parse_duration("1h").unwrap();
    assert_eq!(d, Duration::hours(1));
    let d = parse_duration("14h").unwrap();
    assert_eq!(d, Duration::hours(14));
    let d = parse_duration("1d").unwrap();
    assert_eq!(d, Duration::days(1));
    let d = parse_duration("10d").unwrap();
    assert_eq!(d, Duration::days(10));
    let d = parse_duration("1w").unwrap();
    assert_eq!(d, Duration::days(7));
    let d = parse_duration("7w").unwrap();
    assert_eq!(d, Duration::days(49));
  }
}
