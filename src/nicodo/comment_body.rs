use crate::{nicodo::info::CommentThread, Wayback};

use super::Info;
use serde::Serialize;

#[derive(Debug, Serialize)]
enum Element {
    #[serde(rename = "ping")]
    Ping(Ping),
    #[serde(rename = "thread")]
    Thread(Thread),
    #[serde(rename = "thread_leaves")]
    ThreadLeaves(Thread),
}

#[derive(Debug, Serialize)]
struct Ping {
    content: String,
}

#[derive(Debug, Serialize, Clone)]
struct Thread {
    thread: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    language: usize,
    user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    with_global: Option<usize>,
    scores: usize,
    nicoru: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    res_from: Option<isize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fork: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    userkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    threadkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    force_184: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    waybackkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    when: Option<i64>,
}

#[derive(Debug)]
pub struct Options<'a, 'b, 'c, 'd> {
    pub info: &'a Info,
    // pub counter_rs: usize,
    // pub counter_ps: usize,
    pub wayback: Option<WaybackOptions<'b>>,
    pub official: Option<OfficialOptions<'c, 'd>>,
}

#[derive(Debug)]
pub struct WaybackOptions<'a> {
    pub waybackkey: &'a str,
    pub wayback: chrono::NaiveDateTime,
}

#[derive(Debug)]
pub struct OfficialOptions<'a, 'b> {
    pub threadkey: &'a str,
    pub force_184: &'b str,
}

pub fn get_body(opts: Options, wayback: &Wayback) -> String {
    let rs = if opts.wayback.is_some() { 2 } else { 0 };
    let mut body: Vec<Element> = vec![Element::Ping(Ping {
        content: format!("rs:{}", rs),
    })];
    let mut c = if opts.wayback.is_some() { 22 } else { 0 };

    let content = format!(
        "0-{}:{}",
        opts.info.video.duration / 60
            + (if opts.info.video.duration % 60 > 0 {
                1
            } else {
                0
            }),
        if opts.wayback.is_some() {
            "0,1000" // 0:250
        } else {
            "100,1000,nicoru:100"
        }
    );

    let thread = |t: &CommentThread, leaf: bool| Thread {
        thread: t.id.to_string(),
        version: if !leaf {
            if t.is_owner_thread {
                Some("20061206".to_string())
            } else {
                Some("20090904".to_string())
            }
        } else {
            None
        },
        language: 0,
        fork: Some(t.fork),
        user_id: opts.info.viewer.id.to_string(),
        with_global: if leaf { None } else { Some(1) },
        scores: 1,
        nicoru: 3,
        res_from: if t.is_owner_thread { Some(-1000) } else { None },
        threadkey: if let Some(w) = opts.official.as_ref() {
            Some(w.threadkey.to_string())
        } else if t.is_thread_key_required {
            t.thread_key.clone()
        } else {
            None
        },
        force_184: if let Some(w) = opts.official.as_ref() {
            Some(w.force_184.to_string())
        } else if t.is_184_forced.unwrap_or(false) {
            Some("1".to_string())
        } else {
            None
        },
        content: if leaf {
            Some(content.to_string())
        } else {
            None
        },
        userkey: if t.is_thread_key_required || opts.wayback.is_some() {
            None
        } else {
            if let Some(_) = opts.wayback {
                None
            } else {
                Some(opts.info.comment.keys.user_key.to_string())
            }
        },
        waybackkey: if let Some(w) = opts.wayback.as_ref() {
            Some(w.waybackkey.to_string())
        } else {
            None
        },
        when: if let Some(w) = opts.wayback.as_ref() {
            Some(w.wayback.timestamp())
        } else {
            None
        },
    };

    body.extend(
        opts.info
            .comment
            .threads
            .iter()
            .filter(|t| {
                t.is_active
                    && ((!wayback.is_wayback() && !opts.wayback.is_some())
                        || (wayback.is_wayback() && opts.wayback.is_some())
                        || t.is_thread_key_required)
            })
            .flat_map(|t| {
                let mut threads: Vec<Element> = vec![];

                threads.push(Element::Ping(Ping {
                    content: format!("ps:{}", c),
                }));

                threads.push(Element::Thread(thread(t, false)));

                threads.push(Element::Ping(Ping {
                    content: format!("pf:{}", c),
                }));

                c += 1;

                if t.is_leaf_required {
                    threads.push(Element::Ping(Ping {
                        content: format!("ps:{}", c),
                    }));

                    threads.push(Element::ThreadLeaves(thread(t, true)));

                    threads.push(Element::Ping(Ping {
                        content: format!("pf:{}", c),
                    }));

                    c += 1;
                }
                threads.into_iter()
            }),
    );

    body.push(Element::Ping(Ping {
        content: format!("rf:{}", rs),
    }));

    let content = serde_json::to_string(&body).unwrap();

    content
}
