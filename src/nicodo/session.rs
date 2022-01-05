use super::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_THREAD: Regex = Regex::new(r"^threadkey=(.+?)&force_184=(.+?)$").unwrap();
    static ref RE_WAYBACK: Regex = Regex::new(r"^waybackkey=(.+?)$").unwrap();
}
#[derive(Debug)]
pub struct Session {
    pub cookie: String,
    pub client: reqwest::Client,
}

impl Session {
    pub fn from_user_session(user_session: &str) -> Self {
        Self {
            cookie: format!("user_session={}", user_session),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_cookie(cookie: &str) -> Self {
        Self {
            cookie: cookie.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub(crate) fn get<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .header(reqwest::header::COOKIE, &self.cookie)
    }

    pub async fn get_thread_key(&self, id: &str) -> Result<(String, String)> {
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
            .ok_or(super::Error::InvalidKey)?;

        Ok((key[1].to_string(), key[2].to_string()))
    }

    pub async fn get_waybackkey(&self, id: &str) -> Result<String> {
        let url = format!("https://flapi.nicovideo.jp/api/getwaybackkey?thread={}", id);
        let res = self
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let key = RE_WAYBACK
            .captures_iter(&res)
            .next()
            .ok_or(super::Error::InvalidKey)?;

        Ok(key[1].to_string())
    }
}
