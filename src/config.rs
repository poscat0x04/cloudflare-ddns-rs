use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub credential: Credentials,
    pub config: DomainConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainConfig {
    pub name: String,
    pub iface: String,
    pub zone_id: String,
    #[serde(default = "default_false")]
    pub dualstack: bool,
    #[serde(default = "default_false")]
    pub proxied: bool,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

fn default_ttl() -> u32 {
    120
}
fn default_false() -> bool {
    false
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    AuthKey { email: String, key: String },
    AuthToken { token: String },
}
