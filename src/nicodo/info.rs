use super::{Error, Result, Session};
use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Info {
    pub video: Video,
    pub comment: Comment,
    pub client: Client,
    pub viewer: Viewer,
}

#[derive(Debug, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub duration: usize,
    #[serde(rename = "registeredAt", with = "registered_at")]
    pub registered_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub keys: CommentKeys,
    pub threads: Vec<CommentThread>,
}

impl Comment {
    pub fn thread_id<'a>(&'a self) -> Option<String> {
        self.threads
            .iter()
            .find(|t| t.is_default_post_target)
            .map(|t| t.id.to_string())
    }

    pub fn is_thread_key_required<'a>(&'a self) -> bool {
        self.threads.iter().all(|t| t.is_thread_key_required)
    }

    pub fn thread_key_required_thread<'a>(&'a self) -> Option<&CommentThread> {
        self.threads.iter().find(|t| t.is_thread_key_required)
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentKeys {
    #[serde(rename = "userKey")]
    pub user_key: String,
}

#[derive(Debug, Deserialize)]
pub struct CommentThread {
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
    #[serde(rename = "threadkey")]
    pub thread_key: Option<String>,
    #[serde(rename = "is184Forced")]
    pub is_184_forced: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Client {
    #[serde(rename = "watchId")]
    pub watch_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Viewer {
    pub id: usize,
}

impl Session {
    pub async fn get_info(&self, id: &str) -> Result<Info> {
        lazy_static! {
            static ref SELECTOR: Selector = Selector::parse("[id=js-initial-watch-data]").unwrap();
        }

        let url = format!("https://www.nicovideo.jp/watch/{}", id);
        let res = self
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let doc = Html::parse_document(&res);
        let data = doc
            .select(&SELECTOR)
            .next()
            .and_then(|n| n.value().attr("data-api-data"))
            .ok_or(Error::InvalidWatchPage)?;

        let info = serde_json::from_str::<Info>(&data).map_err(|err| Error::InvalidInfo(err))?;

        if info.comment.keys.user_key.is_empty() {
            return Err(Error::NotAuthorized);
        }

        Ok(info)
    }
}

mod registered_at {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S+09:00";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
