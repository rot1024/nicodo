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
}
