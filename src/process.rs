use crate::{datetime, error};
use chrono::NaiveDateTime;
use std::{path::Path, str::FromStr};
use tokio::task::spawn_blocking;

const DISPLAY_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const FILENAME_DATETIME_FORMAT: &str = "%Y%m%d%H%M%S";

pub struct Options {
    pub quiet: bool,
    pub session: nicodo::Session,
    pub timespan: Timespan,
    pub format: Format,
    pub output: String,
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

pub async fn process(id: &str, opts: &Options) -> error::Result<()> {
    let info = opts.session.get_info(id).await?;

    let wayback = opts.timespan.wayback(info.video.posted_date_time);

    if !opts.quiet {
        eprintln!("Video: {}", id);
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

    let comments = opts
        .session
        .get_comments(&info, &wayback, |p| {
            if opts.quiet {
                return;
            }
            if let Some(dt) = p {
                eprintln!(
                    "Fetching comments at {}",
                    dt.format(DISPLAY_DATETIME_FORMAT)
                );
            } else {
                eprintln!("Fetching latest comments");
            }
        })
        .await?;

    if !opts.quiet {
        eprintln!("Writing comments to the file");
    }

    let dest = Path::new(&opts.output).join(format!(
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
    ));

    let format = opts.format.clone();

    spawn_blocking(move || -> crate::error::Result<()> {
        let mut file =
            std::fs::File::create(&dest).map_err(|err| crate::error::Error::Write(err))?;
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
    .map_err(|e| error::Error::Error(Box::new(e)))?
}
