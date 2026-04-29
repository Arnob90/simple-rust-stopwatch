use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Logging {
    pub path: PathBuf,
    pub filename_format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub logging_options: Option<Logging>,
}

impl Default for Config {
    fn default() -> Self {
        let logging_default_path = ProjectDirs::from("com", "Arnob", "simple-rust-stopwatch")
            .and_then(|proj| proj.state_dir().map(|p| p.to_path_buf()));

        Config {
            logging_options: logging_default_path.map(|path| Logging {
                path,
                filename_format: "stopwatch_%Y-%m-%d_%H-%M-%S.log".to_string(),
            }),
        }
    }
}

pub fn get_config() -> Config {
    confy::load("simple-rust-stopwatch", Some("simple_rust_stopwatch")).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}. Using default.", e);
        Config::default()
    })
}
