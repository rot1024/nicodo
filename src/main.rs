use clap::{crate_authors, crate_description, crate_name, crate_version};
use error::{Error, Result};
use std::path::Path;

mod config;
mod datetime;
mod error;

#[tokio::main]
async fn main() {
    if let Err(err) = main2().await {
        eprintln!("{}", err);
    }
}

const DISPLAY_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const FILENAME_DATETIME_FORMAT: &str = "%Y%m%d%H%M%S";

async fn main2() -> Result<()> {
    let matcher = clap::app_from_crate!()
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        // .arg(
        //     clap::Arg::with_name("email")
        //         .short("e")
        //         .long("email")
        //         .empty_values(false)
        //         .takes_value(true)
        //         .requires("password"),
        // )
        // .arg(
        //     clap::Arg::with_name("password")
        //         .short("p")
        //         .long("password")
        //         .empty_values(false)
        //         .takes_value(true)
        //         .requires("email"),
        // )
        .arg(
            clap::Arg::with_name("nosaveconfig")
                .long("nosave")
                .help("does not save config file"),
        )
        .arg(
            clap::Arg::with_name("session")
                .short("u")
                .long("session")
                .help("user_session value in cookie")
                .empty_values(false)
                .takes_value(true)
                .conflicts_with_all(&["email", "password"]),
        )
        .arg(
            clap::Arg::with_name("format")
                .short("f")
                .long("format")
                .default_value("xml")
                .possible_value("json")
                .possible_value("xml"),
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .long("output")
                .help("output directory path")
                .default_value(".")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("date")
                .short("d")
                .long("date")
                .help("date: 2019-01-01, 2019-01-01 12:00:00")
                .empty_values(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("period start")
                .short("s")
                .long("start")
                .help("period start: 2019-01-01, 2019-01-01 12:00:00, posted, posted+1d, posted+1w")
                .empty_values(false)
                .takes_value(true)
                .requires_all(&["period end", "interval"])
                .conflicts_with("date"),
        )
        .arg(
            clap::Arg::with_name("period end")
                .short("e")
                .long("end")
                .help("period end: 2019-01-01, 2019-01-01 12:00:00, latest, posted+1d, posted+1w")
                .empty_values(false)
                .takes_value(true)
                .requires_all(&["period start", "interval"])
                .conflicts_with("date"),
        )
        .arg(
            clap::Arg::with_name("interval")
                .short("i")
                .long("interval")
                .help("interval: 1h, 1d")
                .empty_values(false)
                .takes_value(true)
                .requires_all(&["period start", "period end"])
                .conflicts_with("date"),
        )
        .arg(
            clap::Arg::with_name("include latest")
                .short("l")
                .long("latest")
                .help("include latest comments")
                .requires_all(&["period start", "period end"])
                .conflicts_with("date"),
        )
        .arg(
            clap::Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("hide progress"),
        )
        .arg(
            clap::Arg::with_name("id")
                .required(true)
                .help("Video id: www.nicovideo.jp/watch/XXXXXXX"),
        )
        .get_matches();

    let conf = config::Config::load().await;

    let session = if let Some(user_session) = matcher.value_of("session") {
        nicodo::Session::from_user_session(user_session)
    } else if let Some(c) = &conf {
        nicodo::Session::from_cookie(&c.session)
    } else {
        return Err(Error::UserSessionMustBeSpecified);
    };

    if !matcher.is_present("nosaveconfig") {
        if let None = &conf {
            config::Config {
                session: session.cookie.to_string(),
            }
            .save()
            .await?;
        }
    }

    let quite = matcher.is_present("quite");

    if !quite {
        eprintln!("Fetching video info");
    }

    let id = matcher
        .value_of("id")
        .map(|id| id.replace("https://www.nicovideo.jp/watch/", ""))
        .unwrap();

    let info = session.get_info(&id).await?;

    let interval = matcher.value_of("interval");
    let wayback = if let Some(ps) = matcher.value_of("date") {
        nicodo::Wayback::DateTime(datetime::parse_datetime(ps, info.video.posted_date_time)?)
    } else if let None = matcher.value_of("period start") {
        nicodo::Wayback::Latest
    } else {
        let start = datetime::parse_datetime(
            matcher.value_of("period start").unwrap(),
            info.video.posted_date_time,
        )?;
        let end = datetime::parse_datetime(
            matcher.value_of("period end").unwrap(),
            info.video.posted_date_time,
        )?;
        if start >= end {
            return Err(Error::Period);
        }
        let interval = datetime::parse_duration(interval.unwrap())?;
        let include_latest = matcher.is_present("include latest");
        nicodo::Wayback::Period {
            start: start,
            end: end,
            interval: interval,
            include_latest: include_latest,
        }
    };

    if !quite {
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
                interval.unwrap(),
                if include_latest {
                    " (includes latest)"
                } else {
                    ""
                }
            ),
        };
    }

    let comments = session
        .get_comments(&info, &wayback, |p| {
            if quite {
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

    if !quite {
        eprintln!("Writing comments to the file");
    }

    let ext = if let Some("json") = matcher.value_of("format") {
        "json"
    } else {
        "xml"
    };
    let dest = Path::new(matcher.value_of("output").unwrap()).join(format!(
        "{}{}.{}",
        info.video.title,
        match wayback {
            nicodo::Wayback::DateTime(dt) => format!("_{}", dt.format(DISPLAY_DATETIME_FORMAT)),
            nicodo::Wayback::Period {
                start,
                end,
                include_latest,
                ..
            } => format!(
                "_{}-{}_{}{}",
                start.format(FILENAME_DATETIME_FORMAT),
                end.format(FILENAME_DATETIME_FORMAT),
                interval.unwrap(),
                if include_latest { "+l" } else { "" }
            ),
            _ => "".to_string(),
        },
        ext
    ));

    // TODO: asynchronize
    let mut file = std::fs::File::create(&dest).map_err(|err| Error::Write(err))?;

    match ext {
        "json" => {
            nicodo::write_json(&mut file, &comments)?;
        }
        _ => {
            nicodo::write_xml(&mut file, &comments)?;
        }
    }

    if !quite {
        eprintln!("Done!");
    }

    Ok(())
}
