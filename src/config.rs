use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub dir: PathBuf,
    pub openclaw_bin: String,
    pub timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            dir: env_or("SIMPLE_TASK_DIR", default_dir()).into(),
            openclaw_bin: env_or("SIMPLE_TASK_OPENCLAW_BIN", "openclaw".to_owned()),
            timeout_secs: env_parse("SIMPLE_TASK_TIMEOUT_SECS", 3600)?,
        })
    }
}

fn default_dir() -> String {
    let config_home = env_or(
        "XDG_CONFIG_HOME",
        format!("{}/.config", env_or("HOME", ".".to_owned())),
    );
    format!("{config_home}/simple-task")
}

fn env_or(key: &str, default: String) -> String {
    std::env::var(key).unwrap_or(default)
}

fn env_parse<T>(key: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    match std::env::var(key) {
        Err(_) => Ok(default),
        Ok(raw) => raw
            .parse()
            .map_err(|e| Error::Config(format!("{key}: {e}"))),
    }
}
