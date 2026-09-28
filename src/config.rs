use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            dir: env_or("SIMPLE_TASK_DIR", default_dir()).into(),
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
