use super::{
    comment_body::{get_body, Options},
    Error, Info, Result, Session, Wayback,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_ENDPOINT: &'static str = "https://nvcomment.nicovideo.jp/legacy/api.json";

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
    pub async fn get_comments<F: Fn(Option<NaiveDateTime>)>(
        &self,
        info: &Info,
        wayback: &Wayback,
        on_progress: F,
    ) -> Result<Vec<Comment>> {
        let (threadkey, force_184) = self.get_threadkey(&info.client.watch_id).await?;
        let waybackkey = self.get_waybackkey(&info.client.watch_id).await?;

        let mut comments: HashMap<usize, Comment> = HashMap::new();

        for wayback in wayback.iter() {
            on_progress(wayback);

            let (body, _counter_rs, _counter_ps) = get_body(Options {
                info,
                threadkey: &threadkey,
                waybackkey: &waybackkey,
                force_184: &force_184,
                counter_rs: 0,
                counter_ps: 0,
                wayback,
            });

            let res = reqwest::Client::new()
                .post(API_ENDPOINT)
                .body(body)
                .header(reqwest::header::CONTENT_TYPE, "text/plain;charset=UTF-8")
                .timeout(Duration::from_secs(10))
                .send()
                .await?
                .error_for_status()?
                .json::<Vec<Element>>()
                .await?;

            res.into_iter()
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
                .for_each(|c| {
                    comments.insert(c.no, c);
                })
        }

        if comments.len() == 0 {
            return Err(Error::NoComments);
        }

        let mut comments: Vec<_> = comments.into_iter().map(|(_, c)| c).collect();
        comments.sort_by(|a, b| a.vpos.cmp(&b.vpos));

        Ok(comments)
    }
}
