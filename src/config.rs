use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub if_name: String,
    pub zone_id: String,
    pub wait_duration: Option<u64>,
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

#[cfg(test)]
mod test {
    use toml::from_str;

    use crate::config::Config;

    #[test]
    fn test_parse() {
        const SAMPLE_FILE: &str = r#"
name = "a.example.com"
if_name = "ppp"
zone_id = "deadbeaf"
"#;
        let cfg: Result<Config, _> = from_str(SAMPLE_FILE);
        assert!(cfg.is_ok());
        assert!(cfg.unwrap().wait_duration.is_none());
    }
}