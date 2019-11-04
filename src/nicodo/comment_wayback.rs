use chrono::{Duration, NaiveDateTime};

#[derive(Debug, Clone)]
pub enum Wayback {
  Latest,
  DateTime(NaiveDateTime),
  Period {
    start: NaiveDateTime,
    end: NaiveDateTime,
    interval: Duration,
    include_latest: bool,
  },
}

impl Wayback {
  pub fn iter(&self) -> WaybackIter {
    WaybackIter {
      wayback: self.clone(),
      counter: 0,
      duration: Duration::zero(),
      latest: false,
    }
  }
}

impl IntoIterator for Wayback {
  type Item = Option<NaiveDateTime>;
  type IntoIter = WaybackIter;

  fn into_iter(self) -> Self::IntoIter {
    WaybackIter {
      wayback: self,
      counter: 0,
      duration: Duration::zero(),
      latest: false,
    }
  }
}

pub struct WaybackIter {
  wayback: Wayback,
  counter: usize,
  duration: Duration,
  latest: bool,
}

impl Iterator for WaybackIter {
  type Item = Option<NaiveDateTime>;

  fn next(&mut self) -> Option<Self::Item> {
    let res = match self.wayback {
      Wayback::Latest => {
        if self.counter == 0 {
          Some(None)
        } else {
          None
        }
      }
      Wayback::DateTime(dt) => {
        if self.counter == 0 {
          Some(Some(dt))
        } else {
          None
        }
      }
      Wayback::Period {
        start,
        end,
        interval,
        include_latest,
      } => {
        if start >= end || interval.is_zero() {
          None
        } else {
          let d = start + self.duration;
          self.duration = self.duration + interval;
          if d > end {
            if include_latest && !self.latest {
              self.latest = true;
              Some(None)
            } else {
              None
            }
          } else {
            Some(Some(d))
          }
        }
      }
    };
    self.counter += 1;
    res
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{Duration, NaiveDateTime};
  use std::str::FromStr;

  #[test]
  fn test_latest() {
    let mut iter = Wayback::Latest.into_iter();
    assert_eq!(iter.next().unwrap(), None);
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn test_datetime() {
    let dt = NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap();
    let mut iter = Wayback::DateTime(dt).into_iter();
    assert_eq!(iter.next().unwrap(), Some(dt));
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn test_period() {
    let mut iter = Wayback::Period {
      start: NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap(),
      end: NaiveDateTime::from_str("2019-11-10T00:00:00").unwrap(),
      interval: Duration::days(1),
      include_latest: false,
    }
    .into_iter();
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-04T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-05T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-06T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-07T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-08T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-09T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-10T00:00:00").unwrap())
    );
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn test_period2() {
    let mut iter = Wayback::Period {
      start: NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap(),
      end: NaiveDateTime::from_str("2019-11-04T00:00:00").unwrap(),
      interval: Duration::hours(6),
      include_latest: true,
    }
    .into_iter();
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-03T06:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-03T12:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-03T18:00:00").unwrap())
    );
    assert_eq!(
      iter.next().unwrap(),
      Some(NaiveDateTime::from_str("2019-11-04T00:00:00").unwrap())
    );
    assert_eq!(iter.next().unwrap(), None);
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn test_period_error() {
    let mut iter = Wayback::Period {
      start: NaiveDateTime::from_str("2019-11-10T00:00:00").unwrap(),
      end: NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap(),
      interval: Duration::days(1),
      include_latest: false,
    }
    .into_iter();
    assert_eq!(iter.next(), None);
    let mut iter = Wayback::Period {
      start: NaiveDateTime::from_str("2019-11-03T00:00:00").unwrap(),
      end: NaiveDateTime::from_str("2019-11-10T00:00:00").unwrap(),
      interval: Duration::zero(),
      include_latest: false,
    }
    .into_iter();
    assert_eq!(iter.next(), None);
  }
}
