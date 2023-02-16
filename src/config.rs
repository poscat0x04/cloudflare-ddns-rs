use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub if_name: String,
    pub zone_id: String,
    #[serde(default = "default_true")]
    pub v4: bool,
    #[serde(default = "default_true")]
    pub v6: bool,
    #[serde(default = "default_false")]
    pub proxied: bool,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

#[inline]
fn default_ttl() -> u32 { 120 }

#[inline]
fn default_false() -> bool { false }

#[inline]
fn default_true() -> bool { true }
