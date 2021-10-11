use clap::Clap;
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

#[derive(Debug, Clap)]
struct Opts {
    // #[clap(long)]
    // email: String,
    // #[clap(long)]
    // password: String,
    #[clap(short = 'u', long, about = "user_session value in cookie")]
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
    #[clap(long = "reset", short, about = "Reset config")]
    reset_config: bool,
    #[clap(short = 'l', long = "latest", about = "Include latest comments")]
    includes_latest: bool,
    #[clap(short, long, about = "Hide progress")]
    quiet: bool,
    #[clap(long, about = "Dump session ID")]
    dump_session_id: bool,
    #[clap(about = "Video ID, video URL, or channel URL")]
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
    };

    for item in opts.ids {
        process::process(&item, &options).await?;
    }

    if !quiet {
        eprintln!("Done!");
    }

    Ok(())
}
