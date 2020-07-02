use clap::Clap;
use error::{Error, Result};

mod config;
mod datetime;
mod error;
mod process;

#[tokio::main]
async fn main() {
    if let Err(err) = main2().await {
        eprintln!("{}", err);
    }
}

#[derive(Debug, Clap)]
struct Opts {
    // #[clap(long)]
    // email: String,
    // #[clap(long)]
    // password: String,
    #[clap(short = "u", long, about = "user_session value in cookie")]
    session: Option<String>,
    #[clap(short, long, default_value = "xml", about = "Format")]
    format: process::Format,
    #[clap(short, long, about = "Output directory path", default_value = ".")]
    output: String,
    #[clap(short, long, about = "Date: 2019-01-01, 2019-01-01 12:00:00")]
    date: Option<datetime::DateTime>,
    #[clap(
        short,
        long,
        about = "Period start: 2019-01-01, 2019-01-01 12:00:00, posted, posted+1d, posted+1w"
    )]
    start: Option<datetime::DateTime>,
    #[clap(
        short,
        long,
        about = "Period end: 2019-01-01, 2019-01-01 12:00:00, latest, posted+1d, posted+1w"
    )]
    end: Option<datetime::DateTime>,
    #[clap(short, long, about = "Interval: 1h, 1d")]
    interval: Option<datetime::Duration>,
    #[clap(long = "nosave", about = "Config file won't be saved")]
    nosaveconfig: bool,
    #[clap(short = "l", long = "latest", about = "Include latest comments")]
    includes_latest: bool,
    #[clap(short, long, about = "Hide progress")]
    quiet: bool,
    #[clap(long, about = "Dump session ID")]
    dump_session_id: bool,
    #[clap(about = "Video id: www.nicovideo.jp/watch/XXXXXXX")]
    ids: Vec<String>,
}

impl Opts {
    fn timespan(&self) -> Result<process::Timespan> {
        if let (Some(s), Some(e), Some(i)) = (
            self.start.as_ref(),
            self.end.as_ref(),
            self.interval.as_ref(),
        ) {
            Ok(process::Timespan::Period {
                start: s.clone(),
                end: e.clone(),
                interval: i.clone(),
                include_latest: self.includes_latest,
            })
        } else if let Some(d) = self.date.as_ref() {
            Ok(process::Timespan::DateTime(d.clone()))
        } else {
            Ok(process::Timespan::Latest)
        }
    }
}

async fn main2() -> Result<()> {
    let opts = Opts::parse();
    let quiet = opts.quiet;

    // check
    if (opts.start.is_some() || opts.end.is_some() || opts.interval.is_some())
        && !(opts.start.is_some() && opts.end.is_some() && opts.interval.is_some())
    {
        return Err(Error::Period);
    }

    let conf = config::Config::load().await;

    if opts.dump_session_id {
        if let Some(c) = conf {
            println!("{}", c.session);
        } else {
            println!("no config file is saved");
        }
        return Ok(());
    }

    let session = opts
        .session
        .as_ref()
        .map(|s| nicodo::Session::from_user_session(&s))
        .or_else(|| conf.as_ref().map(|c| c.session()))
        .ok_or(Error::UserSessionMustBeSpecified)?;

    if opts.session.is_some() && !opts.nosaveconfig {
        if let None = conf {
            config::Config {
                session: session.cookie.to_string(),
            }
            .save()
            .await?;
        }
    }

    let options = process::Options {
        quiet,
        session,
        timespan: opts.timespan()?,
        format: opts.format,
        output: opts.output,
    };

    for id in opts
        .ids
        .iter()
        .map(|id| id.replace("https://www.nicovideo.jp/watch/", ""))
    {
        process::process(&id, &options).await?;
    }

    if !quiet {
        eprintln!("Done!");
    }

    Ok(())
}
