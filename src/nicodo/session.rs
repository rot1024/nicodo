#[derive(Debug)]
pub struct Session {
  pub cookie: String,
  pub client: reqwest::Client,
}

impl Session {
  pub fn from_session_id(session_id: &str) -> Self {
    Self {
      cookie: format!("user_session={}", session_id),
      client: reqwest::Client::new(),
    }
  }

  pub fn from_cookie(cookie: &str) -> Self {
    Self {
      cookie: cookie.to_string(),
      client: reqwest::Client::new(),
    }
  }
}
