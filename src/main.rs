use clap::{crate_authors, crate_description, crate_name, crate_version};
use std::path::Path;

mod config;
mod error;

#[tokio::main]
async fn main() {
    if let Err(err) = main2().await {
        eprintln!("{}", err);
    }
}

async fn main2() -> error::Result<()> {
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
            clap::Arg::with_name("session")
                .short("s")
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
            clap::Arg::with_name("id")
                .required(true)
                .help("Video id: www.nicovideo.jp/watch/XXXXXXX"),
        )
        .get_matches();

    let conf = config::Config::load().await;

    let session = if let Some(user_session) = matcher.value_of("session") {
        nicodo::Session::from_user_session(user_session)
    } else if let Some(c) = conf {
        nicodo::Session::from_cookie(&c.session)
    } else {
        return Err(error::Error::UserSessionMustBeSpecified);
    };

    config::Config {
        session: session.cookie.to_string(),
    }
    .save()
    .await?;

    let info = session.get_info(matcher.value_of("id").unwrap()).await?;
    let comments = session.get_comments(&info).await?;

    let ext = if let Some("json") = matcher.value_of("format") {
        "json"
    } else {
        "xml"
    };
    let dest = Path::new(matcher.value_of("output").unwrap())
        .join(format!("{}.{}", info.video.title, ext));

    // TODO: asynchronize
    let mut file = std::fs::File::create(&dest).map_err(|err| error::Error::Write(err))?;

    match ext {
        "json" => {
            nicodo::write_json(&mut file, &comments)?;
        }
        _ => {
            nicodo::write_xml(&mut file, &comments)?;
        }
    }

    Ok(())
}
