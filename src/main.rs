use clap::{crate_authors, crate_description, crate_name, crate_version};

#[tokio::main]
async fn main() {
    let _matcher = clap::app_from_crate!()
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .get_matches();
}
