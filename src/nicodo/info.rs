use super::{Error, Result, Session};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Info {
  #[serde(rename = "commentComposite")]
  pub comment_composite: CommentComposite,
  pub context: Context,
}

#[derive(Debug, Deserialize)]
pub struct CommentComposite {
  pub threads: Vec<CommentCompositeThread>,
}

#[derive(Debug, Deserialize)]
pub struct CommentCompositeThread {
  pub id: usize,
  pub fork: usize,
  #[serde(rename = "isActive")]
  pub is_active: bool,
  #[serde(rename = "isDefaultPostTarget")]
  pub is_default_post_target: bool,
  #[serde(rename = "isThreadkeyRequired")]
  pub is_thread_key_required: bool,
  #[serde(rename = "isLeafRequired")]
  pub is_leaf_required: bool,
  pub label: String,
  #[serde(rename = "isOwnerThread")]
  pub is_owner_thread: bool,
}

#[derive(Debug, Deserialize)]
pub struct Context {
  pub userkey: String,
}

impl Session {
  pub async fn get_info(&self, id: &str) -> Result<Info> {
    let url = format!("https://www.nicovideo.jp/watch/{}", id);
    let res = self
      .client
      .get(&url)
      .header(reqwest::header::COOKIE, &self.cookie)
      .send()
      .await?
      .error_for_status()?
      .text()
      .await?;

    let doc = select::document::Document::from(&res as &str);
    let data = doc
      .find(select::predicate::Attr("id", "js-initial-watch-data"))
      .next()
      .and_then(|n| n.attr("data-api-data"))
      .ok_or(Error::InvalidWatchPage)?;

    let info = serde_json::from_str::<Info>(data).map_err(|err| Error::InvalidInfo(err))?;

    Ok(info)
  }
}
