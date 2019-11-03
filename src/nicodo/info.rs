use super::{Error, Result, Session};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Info {
  pub video: Video,
  #[serde(rename = "commentComposite")]
  pub comment_composite: CommentComposite,
  pub context: Context,
  pub viewer: Viewer,
}

#[derive(Debug, Deserialize)]
pub struct Video {
  pub id: String,
  pub title: String,
  pub duration: usize,
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
  #[serde(rename = "isOwnerThread")]
  pub is_owner_thread: bool,
}

#[derive(Debug, Deserialize)]
pub struct Context {
  pub userkey: String,
  #[serde(rename = "watchId")]
  pub watch_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Viewer {
  pub id: usize,
}

impl Session {
  pub async fn get_info(&self, id: &str) -> Result<Info> {
    let url = format!("https://www.nicovideo.jp/watch/{}", id);
    let res = self
      .get(&url)
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

    if info.context.userkey.is_empty() {
      return Err(Error::NotAuthorized);
    }

    Ok(info)
  }
}
