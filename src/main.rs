use anyhow::Result;
use clap::{clap_app, crate_authors, crate_description, crate_version, App};

mod agent;
mod config;
mod server;

fn app_setup() -> App<'static, 'static> {
    clap_app!(axum_websockify =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg debug: -d --debug "Sets log_level to debug")
        (@arg log_level: -l --log_level +takes_value "One of (error, warn[default], info, debug, trace) \
        Note this value will overwrite -d settings")
        (@arg cert: --cert +takes_value "SSL certificate file")
        (@arg key: --key +takes_value "SSL key file")
        (@arg web: --web +takes_value +required "Serve files from <web>")
        (@arg src_addr: +takes_value +required "[source_addr:]source_port")
        (@arg dst_addr: +takes_value +required "target_addr:target_port")
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = app_setup();
    let matches = app.get_matches();

    config::init_from_matches(&matches)?;

    server::run().await
}
