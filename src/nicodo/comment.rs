use super::{
  comment_body::{get_body, Options},
  Info, Result, Session,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Element {
  chat: Option<Chat>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
  thread: String,
  no: usize,
  vpos: usize,
  leaf: Option<usize>,
  date: usize,
  date_usec: Option<usize>,
  anonymity: usize,
  user_id: Option<String>,
  mail: Option<String>,
  content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Comment {
  pub thread: String,
  pub no: usize,
  pub vpos: usize,
  pub date: usize,
  pub user_id: Option<String>,
  pub content: String,
  pub mail: Option<String>,
}

impl Session {
  pub async fn get_comments(&self, info: &Info) -> Result<Vec<Comment>> {
    let (threadkey, force_184) = self.get_threadkey(&info.context.watch_id).await?;

    let (body, _counter_rs, _counter_ps) = get_body(Options {
      info: info,
      threadkey: &threadkey,
      force_184: &force_184,
      counter_rs: 0,
      counter_ps: 0,
    });

    let res = reqwest::Client::new()
      .post("https://nmsg.nicovideo.jp/api.json/")
      .body(body)
      .header(reqwest::header::CONTENT_TYPE, "text/plain;charset=UTF-8")
      .send()
      .await?
      .error_for_status()?
      .json::<Vec<Element>>()
      .await?;

    let mut comments = res
      .into_iter()
      .filter_map(|e| e.chat)
      .filter(|c| c.content.is_some())
      .map(|c| Comment {
        thread: c.thread,
        no: c.no,
        vpos: c.vpos,
        date: c.date,
        user_id: c.user_id,
        content: c.content.unwrap(),
        mail: c.mail,
      })
      .collect::<Vec<_>>();

    comments.sort_by(|a, b| a.vpos.cmp(&b.vpos));

    Ok(comments)
  }
}
