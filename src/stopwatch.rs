use anyhow::Context;
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use humantime::Duration;
use std::path::Path;
use std::str::FromStr;
use std::{path::PathBuf, time};
use thiserror::Error;
use tokio::fs;
use tracing::warn;
pub struct Stopwatch {
    starting_time: time::Instant,
}

impl Stopwatch {
    pub fn start_from(time: time::Instant) -> Stopwatch {
        Stopwatch {
            starting_time: time,
        }
    }
    pub fn get_time(&self) -> time::Duration {
        self.starting_time.elapsed()
    }
}

#[derive(Debug, Error)]
pub enum ResumptionError {
    #[error("Failed to resume file")]
    OpenFileError(#[from] std::io::Error),
    #[error("Failed to parse time")]
    ParseError(#[from] humantime::DurationError),
}
pub async fn get_time_from_file(filepath: &Path) -> Result<std::time::Duration, ResumptionError> {
    let text = fs::read_to_string(filepath).await?;
    let trimmed = text.trim();
    let time = trimmed.parse::<humantime::Duration>()?;
    Ok(time.into())
}

#[derive(Debug, Subcommand)]
pub enum Mode {
    Start {
        #[arg(long, short)]
        time: Option<String>,
    },
    Resume {
        #[arg(long, short, conflicts_with = "offset")]
        file: Option<PathBuf>,
        #[arg(long, short)]
        offset: Option<usize>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about = "A simple CLI stopwatch that saves your progress.", long_about = None)]
pub struct Args {
    /// Optional starting time (e.g., "1h 30m 10s").
    /// If provided, the stopwatch will count up from this duration.
    /// Uses humantime format
    #[command(subcommand)]
    command: Option<Mode>,
}
#[derive(Error, Debug)]
pub enum FilenameGetError {
    #[error("Log path unreadable error {0}")]
    LogPathUnreadableError(#[from] std::io::Error),
}
pub async fn get_files_in_log_path(log_path: &Path) -> Result<Vec<PathBuf>, FilenameGetError> {
    let mut entries = Vec::new();
    let mut read_dir = fs::read_dir(log_path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if entry.path().is_file() {
            entries.push(entry.path());
        }
    }
    Ok(entries)
}
pub async fn get_time_from_offset_last(
    offset: usize,
    format_str: &str,
    log_path: &Path,
) -> anyhow::Result<std::time::Duration> {
    // 1. Get all entries and SORT them to ensure chronological order
    let entries = get_files_in_log_path(log_path).await?;
    let filenames: Vec<String> = entries
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    let mut date_to_filename: Vec<(NaiveDateTime, PathBuf)> = vec![];
    for filename in filenames {
        let parsed_datetime = NaiveDateTime::parse_from_str(&filename, format_str);
        if let Err(e) = parsed_datetime {
            warn!(
                "Cannot parse {} in {}. Skipping. Reason: {} ",
                &filename,
                log_path.display(),
                e
            );
            continue;
        }
        let parsed_duration = parsed_datetime.unwrap();
        date_to_filename.push((parsed_duration, log_path.join(&filename)));
    }
    date_to_filename.sort_by_key(|(d, _)| *d);
    let last_idx = (date_to_filename.len() - 1) as i64;
    let offset_i = offset as i64;
    let to_pick = (last_idx - offset_i).rem_euclid(date_to_filename.len() as i64);
    let filename = &date_to_filename[to_pick as usize].1;
    let filepath_to_read = log_path.join(filename);
    let parsed = get_time_from_file(&filepath_to_read)
        .await
        .context(format!("Failed to parse: {}", filepath_to_read.display()))?;
    Ok(parsed)
}

pub async fn get_time(
    given_args: Args,
    log_path: &Path,
    format_spec: &str,
) -> anyhow::Result<std::time::Duration> {
    let given_command = given_args.command.unwrap_or(Mode::Start { time: None });
    match given_command {
        Mode::Start { time: time_maybe } => {
            let time_parsed: std::time::Duration = match time_maybe {
                Some(time) => Duration::from_str(&time)
                    .context(format!("Invalid time format {time}, starting from 0"))?
                    .into(),
                None => std::time::Duration::from_secs(0),
            };
            Ok(time_parsed)
        }
        Mode::Resume { file, offset } => {
            if let Some(file) = file {
                return get_time_from_file(&file).await.context(format!(
                    "Failed to get time from file: {0}",
                    file.to_string_lossy()
                ));
            } else if let Some(offset) = offset {
                return get_time_from_offset_last(offset, format_spec, log_path).await;
            } else {
                return get_time_from_offset_last(0, format_spec, log_path).await;
            }
        }
    }
}
