use std::time::Duration;

use anyhow::{anyhow, Context};
use cloudflare::framework::async_api::Client;
use cloudflare::framework::auth::Credentials::UserAuthToken;
use cloudflare::framework::Environment::Production;
use tokio::fs::read_to_string;
use tokio::time::sleep;
use toml::from_str;

use crate::cli::Args;
use crate::config::Config;
use crate::dns::update_dns_with;
use crate::exit_code::{AppResult, ExitCodeError, Result, user_error_exitcode, WithExitCode};
use crate::interface::{Addrs, get_interested_addrs};
use crate::notify::notify_startup_complete;
use crate::periodic::run_periodically;

mod config;
mod cli;
mod dns;
mod interface;
mod notify;
mod periodic;
mod exit_code;

const DEFAULT_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[tokio::main]
async fn main() -> AppResult<(), ExitCodeError> {
    let r = main_().await;
    AppResult(r)
}

async fn main_() -> Result<()> {
    // initialization
    let args: Args = argh::from_env();

    let cfg_file =
        read_to_string(&args.config).await
            .context("Failed to read config file")
            .exit_code(user_error_exitcode())?;
    let cfg: Config =
        from_str(&cfg_file)
            .context("Failed to parse config file")
            .exit_code(user_error_exitcode())?;
    drop(cfg_file);

    if !cfg.v4 && !cfg.v6 {
        Err(anyhow!(r#"The config options "v4" and "v6" cant both be false."#))
            .exit_code(user_error_exitcode())?
    }

    let interval =
        cfg.wait_duration
            .map(|min| Duration::from_secs(min * 60))
            .unwrap_or(DEFAULT_INTERVAL);

    let token =
        read_to_string(&args.token).await
            .context("Failed to read token")
            .exit_code(user_error_exitcode())?
            .trim().to_string();
    let client =
        Client::new(UserAuthToken { token }, Default::default(), Production)
            .context("Failed to build cloudflare API client")?;
    // initialization complete

    do_update(&client, &cfg).await?;

    if args.daemon {
        notify_startup_complete()?;
        println!("Going into sleep for {interval:?}");
        sleep(interval).await;

        run_periodically(interval, || async {
            let r = do_update(&client, &cfg).await;
            println!("Going into sleep for {interval:?}");
            r
        }).await?;
    }

    Ok(())
}

async fn do_update(client: &Client, cfg: &Config) -> Result<()> {
    let Addrs {
        v4_addrs,
        v6_addrs,
    } = get_interested_addrs(&cfg.if_name)?;
    update_dns_with(client, cfg, v4_addrs, v6_addrs).await
}
