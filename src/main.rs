use clap::{crate_authors, crate_description, crate_name, crate_version};

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

    println!("{:?}", comments);

    Ok(())
}
