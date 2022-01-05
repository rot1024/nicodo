use super::{
    comment_body::{get_body, Options, WaybackOptions},
    Error, Info, Result, Session, Wayback,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;

const API_ENDPOINT: &'static str = "https://nvcomment.nicovideo.jp/legacy/api.json";

#[derive(Debug, Deserialize)]
struct Element {
    chat: Option<Chat>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    thread: String,
    no: usize,
    vpos: isize,
    date: isize,
    // leaf: Option<isize>,
    // date_usec: Option<isize>,
    // anonymity: isize,
    user_id: Option<String>,
    mail: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Comment {
    pub thread: String,
    pub no: usize,
    pub vpos: isize,
    pub date: isize,
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
        delay: Option<u64>,
        on_progress: F,
    ) -> Result<Vec<Comment>> {
        let tid = match info.comment.thread_id() {
            Some(tid) => tid,
            None => return Err(Error::InvalidKey),
        };

        let wayback_info = if wayback.is_wayback() {
            let (threadkey, force_184) = self.get_thread_key(&tid).await?;
            let waybackkey = self.get_waybackkey(&tid).await?;
            Some((threadkey, force_184, waybackkey))
        } else {
            None
        };

        let mut comments: HashMap<usize, Comment> = HashMap::new();
        let wayback_iter = wayback.iter();
        let wayback_len = wayback_iter.len();

        for (index, current) in wayback_iter.enumerate() {
            let body = get_body(Options {
                info,
                wayback: current.and_then(|c| {
                    if let Some((threadkey, force_184, waybackkey)) = wayback_info.as_ref() {
                        Some(WaybackOptions {
                            force_184,
                            threadkey,
                            waybackkey,
                            wayback: c,
                        })
                    } else {
                        None
                    }
                }),
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

            if let Some(d) = delay {
                // avoid 429 Too Many Requests
                sleep(Duration::from_secs(d)).await;
            }
        }

        let mut comments: Vec<_> = comments.into_iter().map(|(_, c)| c).collect();
        comments.sort_by(|a, b| a.vpos.cmp(&b.vpos));

        Ok(comments)
    }
}
