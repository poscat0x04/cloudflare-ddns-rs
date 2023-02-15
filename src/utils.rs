use std::default::Default;
use std::env::var;

use anyhow::{Context, Result};
pub use cloudflare::framework::async_api::ApiClient;
use cloudflare::framework::async_api::Client;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::Environment;

pub fn build_client_from_env() -> Result<Client> {
    let token = var("CLOUDFLARE_API_TOKEN")
        .context("Failed to read API token from environment variable.\nPlease make sure that CLOUDFLARE_API_TOKEN is set properly.")?;
    let credential = Credentials::UserAuthToken { token };
    Client::new(credential, Default::default(), Environment::Production)
        .context("Failed to build client")
}
