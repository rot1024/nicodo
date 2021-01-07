use super::{Result, Session};
use lazy_static::lazy_static;
use scraper::{Html, Selector};

pub struct Video {
    pub id: String,
    pub title: String,
}

pub type Channel = Vec<Video>;

impl Session {
    pub async fn get_channel(&self, id: &str) -> Result<Channel> {
        lazy_static! {
            static ref SELECTOR: Selector =
                Selector::parse("a.g-video-link:not(.thumb_anchor)").unwrap();
        }

        let url = format!("https://ch.nicovideo.jp/{}", id);
        let res = self
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let doc = Html::parse_document(&res);
        Ok(doc
            .select(&SELECTOR)
            .filter_map(|n| {
                if let (Some(id), Some(title)) = (n.value().attr("href"), n.value().attr("title")) {
                    Some(Video {
                        id: id.replace("https://www.nicovideo.jp/watch/", ""),
                        title: title.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>())
    }
}
