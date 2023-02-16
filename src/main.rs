use anyhow::{bail, Context, Result};
use cloudflare::framework::async_api::Client;
use tokio::fs::read_to_string;
use toml::from_str;

use crate::cli::Args;
use crate::config::Config;
use crate::dns::update_dns_with;
use crate::iface::{Addrs, get_interested_addrs};
use crate::utils::build_client_from_env;

mod config;
mod cli;
mod dns;
mod iface;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // initialization
    let args: Args = argh::from_env();

    let cfg_file =
        read_to_string(&args.config).await
            .context("Failed to read config file")?;
    let cfg: Config = from_str(&cfg_file).context("Failed to parse config file")?;
    drop(cfg_file);

    if !cfg.v4 && !cfg.v6 { bail!(r#"The config options "v4" and "v6" cant both be false."#) }

    let client = build_client_from_env()?;
    // initialization complete

    do_update(&client, &cfg).await?;

    Ok(())
}

async fn do_update(client: &Client, cfg: &Config) -> Result<()> {
    let Addrs {
        v4_addrs,
        v6_addrs,
    } = get_interested_addrs(&cfg.if_name)?;
    update_dns_with(client, cfg, v4_addrs, v6_addrs).await
}
