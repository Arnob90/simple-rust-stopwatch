use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub path_to_log_time: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let logging_default_path = ProjectDirs::from("com", "Arnob", "simple-rust-stopwatch")
            .and_then(|proj| proj.state_dir().map(|p| p.to_path_buf()));

        Config {
            path_to_log_time: logging_default_path,
        }
    }
}

pub fn get_config() -> Config {
    confy::load("simple-rust-stopwatch", None).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}. Using default.", e);
        Config::default()
    })
}
