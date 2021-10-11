use super::id::Id;
use crate::{datetime, error};
use chrono::NaiveDateTime;
use std::{borrow::Cow, convert::TryInto, path::Path, str::FromStr};
use tokio::task::spawn_blocking;

const DISPLAY_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const FILENAME_DATETIME_FORMAT: &str = "%Y%m%d%H%M%S";

pub struct Options {
    pub quiet: bool,
    pub session: nicodo::Session,
    pub timespan: Timespan,
    pub format: Format,
    pub output: String,
    pub delay: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum Format {
    XML,
    JSON,
}

impl FromStr for Format {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "xml" => Ok(Self::XML),
            "json" => Ok(Self::JSON),
            _ => Err("invalid format"),
        }
    }
}

impl Format {
    fn ext(&self) -> &'static str {
        match self {
            Self::XML => "xml",
            Self::JSON => "json",
        }
    }
}

#[derive(Debug)]
pub enum Timespan {
    DateTime(datetime::DateTime),
    Period {
        start: datetime::DateTime,
        end: datetime::DateTime,
        interval: datetime::Duration,
        include_latest: bool,
    },
    Latest,
}

impl Timespan {
    fn wayback(&self, posted_date_time: NaiveDateTime) -> nicodo::Wayback {
        match self {
            Self::DateTime(d) => nicodo::Wayback::DateTime(d.datetime(posted_date_time)),
            Self::Period {
                start,
                end,
                interval,
                include_latest,
            } => nicodo::Wayback::Period {
                start: start.datetime(posted_date_time),
                end: end.datetime(posted_date_time),
                interval: interval.duration(),
                include_latest: *include_latest,
            },
            Self::Latest => nicodo::Wayback::Latest,
        }
    }

    fn interval(&self) -> Option<&datetime::Duration> {
        if let Self::Period { interval, .. } = self {
            Some(interval)
        } else {
            None
        }
    }
}

pub async fn process(item: &Id, opts: &Options) -> error::Result<()> {
    for id in match item {
        Id::Channel(id) => {
            let res = opts.session.get_channel(id).await?;

            eprintln!("Channel: {} ({} videos)", &id, res.len());

            res.into_iter().map(|v| Cow::from(v.id)).collect::<Vec<_>>()
        }
        Id::Video(id) => vec![Cow::from(id)],
    } {
        process_video(&id, opts).await?;
    }

    Ok(())
}

async fn process_video(id: &str, opts: &Options) -> error::Result<()> {
    let info = opts.session.get_info(id).await?;

    let wayback = opts.timespan.wayback(info.video.registered_at);

    if !opts.quiet {
        eprintln!("Video: {} ({})", id, info.video.title);
        match wayback {
            nicodo::Wayback::Latest => eprintln!("Latest comments"),
            nicodo::Wayback::DateTime(dt) => {
                eprintln!("Comments at {}", dt.format(DISPLAY_DATETIME_FORMAT))
            }
            nicodo::Wayback::Period {
                start,
                end,
                include_latest,
                ..
            } => eprintln!(
                "Period: {} ~ {}, Interval: {}{}",
                start.format(DISPLAY_DATETIME_FORMAT),
                end.format(DISPLAY_DATETIME_FORMAT),
                opts.timespan
                    .interval()
                    .map(|i| i.source())
                    .unwrap_or_default(),
                if include_latest {
                    " (includes latest)"
                } else {
                    ""
                }
            ),
        };
    }

    let progress = if !opts.quiet {
        Some(indicatif::ProgressBar::new(0).with_style(
            indicatif::ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} {msg}"),
        ))
    } else {
        None
    };
    let comments = opts
        .session
        .get_comments(&info, &wayback, opts.delay, |ctx| {
            if let Some(p) = progress.as_ref() {
                if let Some(dt) = ctx.wayback {
                    p.set_length(ctx.total.try_into().unwrap());
                    p.set_position((ctx.progress + 1).try_into().unwrap());
                    p.set_message(format!(
                        "{} ({})",
                        dt.format(DISPLAY_DATETIME_FORMAT),
                        ctx.comments.len()
                    ));
                } else {
                    p.set_message(format!("latest ({})", ctx.comments.len()));
                }
            }
        })
        .await?;

    if let Some(p) = progress.as_ref() {
        p.finish_and_clear();
    }

    let comments_len = comments.len();
    if comments_len == 0 {
        if !opts.quiet {
            eprintln!("No comments fetched");
        }
        return Ok(());
    }

    let filename = format!(
        "{}{}.{}",
        info.video.title,
        match wayback {
            nicodo::Wayback::DateTime(dt) => format!("_{}", dt.format(DISPLAY_DATETIME_FORMAT)),
            nicodo::Wayback::Period {
                start,
                end,
                include_latest,
                interval,
                ..
            } => format!(
                "_{}-{}_{}{}",
                start.format(FILENAME_DATETIME_FORMAT),
                end.format(FILENAME_DATETIME_FORMAT),
                interval,
                if include_latest { "+l" } else { "" }
            ),
            _ => "".to_string(),
        },
        opts.format.ext(),
    );
    let dest = Path::new(&opts.output).join(&filename);
    let format = opts.format.clone();

    spawn_blocking(move || -> crate::error::Result<()> {
        let mut file = std::fs::File::create(&dest)?;
        match format {
            Format::JSON => {
                nicodo::write_json(&mut file, &comments)?;
            }
            Format::XML => {
                nicodo::write_xml(&mut file, &comments)?;
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| error::Error::Error(Box::new(e)))??;

    if !opts.quiet {
        eprintln!("Writing {} comments to \"{}\"", comments_len, filename);
    }

    Ok(())
}
