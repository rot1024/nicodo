use super::{
    comment_body::{get_body, Options},
    Info, Result, Session, Wayback,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

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

#[derive(Debug)]
pub struct Context<'a> {
    pub wayback: Option<NaiveDateTime>,
    pub total: usize,
    pub progress: usize,
    pub comments: &'a [Comment],
}

impl Session {
    pub async fn get_comments<F: Fn(Context)>(
        &self,
        info: &Info,
        wayback: &Wayback,
        on_progress: F,
    ) -> Result<Vec<Comment>> {
        let (threadkey, force_184) = self.get_threadkey(&info.client.watch_id).await?;
        let waybackkey = self.get_waybackkey(&info.client.watch_id).await?;

        let mut comments: HashMap<usize, Comment> = HashMap::new();

        let wayback_iter = wayback.iter();
        let wayback_len = wayback_iter.len();

        for (index, current) in wayback_iter.enumerate() {
            let (body, _counter_rs, _counter_ps) = get_body(Options {
                info,
                threadkey: &threadkey,
                waybackkey: &waybackkey,
                force_184: &force_184,
                counter_rs: 0,
                counter_ps: 0,
                wayback: current,
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

            let current_comments = res
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

            on_progress(Context {
                wayback: current,
                total: wayback_len,
                progress: index,
                comments: &current_comments,
            });

            current_comments.into_iter().for_each(|c| {
                comments.insert(c.no, c);
            });
        }

        let mut comments: Vec<_> = comments.into_iter().map(|(_, c)| c).collect();
        comments.sort_by(|a, b| a.vpos.cmp(&b.vpos));

        Ok(comments)
    }
}
