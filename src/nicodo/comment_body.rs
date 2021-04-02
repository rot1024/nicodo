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
    when: Option<String>,
}

#[derive(Debug)]
pub struct Options<'a, 'b, 'c, 'd> {
    pub info: &'a Info,
    pub threadkey: &'b str,
    pub waybackkey: &'c str,
    pub force_184: &'d str,
    pub counter_rs: usize,
    pub counter_ps: usize,
    pub wayback: Option<chrono::NaiveDateTime>,
}

pub fn get_body(opts: Options) -> (String, usize, usize) {
    let mut c = opts.counter_ps;

    let mut body: Vec<Element> = vec![Element::Ping(Ping {
        content: format!("rs:{}", opts.counter_rs),
    })];

    let content = format!(
        "0-{}:100,1000,nicoru:100",
        opts.info.video.duration / 60
            + (if opts.info.video.duration % 60 > 0 {
                1
            } else {
                0
            })
    );

    body.extend(
        opts.info
            .comment
            .threads
            .iter()
            .filter(|t| t.is_active)
            .flat_map(|t| {
                let mut threads: Vec<Element> = vec![];

                threads.push(Element::Ping(Ping {
                    content: format!("ps:{}", c),
                }));

                threads.push(Element::Thread(Thread {
                    thread: t.id.to_string(),
                    version: if t.is_owner_thread {
                        Some("20061206".to_string())
                    } else {
                        Some("20090904".to_string())
                    },
                    language: 0,
                    fork: Some(t.fork),
                    user_id: opts.info.viewer.id.to_string(),
                    with_global: Some(1),
                    scores: 1,
                    nicoru: 3,
                    res_from: if t.is_owner_thread { Some(-1000) } else { None },
                    threadkey: if t.is_thread_key_required {
                        Some(opts.threadkey.to_string())
                    } else {
                        None
                    },
                    force_184: if t.is_thread_key_required {
                        Some(opts.force_184.to_string())
                    } else {
                        None
                    },
                    content: None,
                    userkey: if t.is_thread_key_required {
                        None
                    } else {
                        if let Some(_) = opts.wayback {
                            None
                        } else {
                            Some(opts.info.comment.keys.user_key.to_string())
                        }
                    },
                    waybackkey: if let Some(_) = opts.wayback {
                        Some(opts.waybackkey.to_string())
                    } else {
                        None
                    },
                    when: if let Some(dt) = opts.wayback {
                        Some(dt.timestamp().to_string())
                    } else {
                        None
                    },
                }));

                threads.push(Element::Ping(Ping {
                    content: format!("pf:{}", c),
                }));

                c += 1;

                if t.is_leaf_required {
                    threads.push(Element::Ping(Ping {
                        content: format!("ps:{}", c),
                    }));

                    threads.push(Element::ThreadLeaves(Thread {
                        thread: t.id.to_string(),
                        version: None, // thread_leaves
                        language: 0,
                        fork: None, // thread_leaves
                        user_id: opts.info.viewer.id.to_string(),
                        with_global: None, // thread_leaves
                        scores: 1,
                        nicoru: 3,
                        res_from: if t.is_owner_thread { Some(-1000) } else { None },
                        threadkey: if t.is_thread_key_required {
                            Some(opts.threadkey.to_string())
                        } else {
                            None
                        },
                        force_184: if t.is_thread_key_required {
                            Some(opts.force_184.to_string())
                        } else {
                            None
                        },
                        content: Some(content.to_string()),
                        userkey: if t.is_thread_key_required {
                            None
                        } else {
                            if let Some(_) = opts.wayback {
                                None
                            } else {
                                Some(opts.info.comment.keys.user_key.to_string())
                            }
                        },
                        waybackkey: if let Some(_) = opts.wayback {
                            Some(opts.waybackkey.to_string())
                        } else {
                            None
                        },
                        when: if let Some(dt) = opts.wayback {
                            Some(dt.timestamp().to_string())
                        } else {
                            None
                        },
                    }));

                    threads.push(Element::Ping(Ping {
                        content: format!("pf:{}", c),
                    }));

                    c += 1;
                }
                threads.into_iter()
            }),
    );

    body.push(Element::Ping(Ping {
        content: format!("rf:{}", opts.counter_rs),
    }));

    (
        serde_json::to_string(&body).unwrap(),
        opts.counter_ps + 1,
        c,
    )
}
