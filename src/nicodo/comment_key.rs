use super::{Error, Result, Session};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  static ref RE_THREAD: Regex = Regex::new(r"^threadkey=(.+?)&force_184=(.+?)$").unwrap();
  static ref RE_WAYBACK: Regex = Regex::new(r"^waybackkey=(.+?)$").unwrap();
}

impl Session {
  pub async fn get_threadkey(&self, id: &str) -> Result<(String, String)> {
    let url = format!("https://flapi.nicovideo.jp/api/getthreadkey?thread={}", id);
    let res = self
      .get(&url)
      .send()
      .await?
      .error_for_status()?
      .text()
      .await?;

    let key = RE_THREAD
      .captures_iter(&res)
      .next()
      .ok_or(Error::InvalidKey)?;

    Ok((key[1].to_string(), key[2].to_string()))
  }

  pub async fn get_waybackkey(&self, id: &str) -> Result<String> {
    let url = format!("https://flapi.nicovideo.jp/api/getwaybackkey?thread={}", id);
    let res = self.get(&url).send().await?.text().await?;

    let key = RE_WAYBACK
      .captures_iter(&res)
      .next()
      .ok_or(Error::InvalidKey)?;

    Ok(key[1].to_string())
  }
}
