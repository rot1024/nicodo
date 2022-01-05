use clap::Parser;
use dialoguer::Input;
use error::{Error, Result};
use std::process::exit;

mod config;
mod datetime;
mod error;
mod id;
mod process;

#[tokio::main]
async fn main() {
    if let Err(err) = main2().await {
        eprintln!("{}", err);
        exit(1);
    }
}

#[derive(Debug, Parser)]
#[clap(about, version, author)]
struct Opts {
    // #[clap(long)]
    // email: String,
    // #[clap(long)]
    // password: String,
    /// user_session value in cookie
    #[clap(short = 'u', long)]
    session: Option<String>,
    /// Format
    #[clap(short, long, default_value = "xml")]
    format: process::Format,
    /// Output directory path
    #[clap(short, long, default_value = ".")]
    output: String,
    /// Date: 2019-01-01, 2019-01-01 12:00:00
    #[clap(short, long)]
    date: Option<datetime::DateTime>,
    /// Period start: 2019-01-01, 2019-01-01 12:00:00, posted, posted+1d, posted+1w
    #[clap(short, long)]
    start: Option<datetime::DateTime>,
    /// Period end: 2019-01-01, 2019-01-01 12:00:00, latest, posted+1d, posted+1w
    #[clap(short, long)]
    end: Option<datetime::DateTime>,
    /// Interval: 1h, 1d
    #[clap(short, long)]
    interval: Option<datetime::Duration>,
    /// Config file won't be saved
    #[clap(long = "nosave")]
    nosaveconfig: bool,
    /// Reset config
    #[clap(long = "reset", short)]
    reset_config: bool,
    /// Include latest comments
    #[clap(short = 'l', long = "latest")]
    includes_latest: bool,
    /// Hide progress
    #[clap(short, long)]
    quiet: bool,
    /// Dump session ID
    #[clap(long)]
    dump_session_id: bool,
    /// Delay (seconds)
    #[clap(long, default_value = "1")]
    delay: u64,
    /// Video ID, video URL, or channel URL
    ids: Vec<id::Id>,
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

    let mut conf = config::Config::load()?;
    let mut conf_changed = false;

    if opts.dump_session_id {
        if !conf.session.is_empty() {
            println!("{}", &conf.session);
        }
        return Ok(());
    }

    if conf.session.is_empty() && opts.session.is_none() || opts.reset_config {
        conf.session = Input::<String>::new()
            .with_prompt("Session (user_session's value)")
            .interact()?;
        conf_changed = true;
    }

    if let Some(s) = opts.session.as_ref() {
        conf.session = s.to_string();
        conf_changed = true;
    }

    if conf_changed && !opts.nosaveconfig {
        conf.save()?;
    }

    let session = nicodo::Session::from_user_session(&conf.session);

    let options = process::Options {
        quiet,
        session,
        timespan: opts.timespan()?,
        format: opts.format,
        output: opts.output,
        delay: Some(opts.delay),
    };

    for item in opts.ids {
        process::process(&item, &options).await?;
    }

    if !quiet {
        eprintln!("Done!");
    }

    Ok(())
}
