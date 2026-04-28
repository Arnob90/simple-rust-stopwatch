mod config;
use chrono::Local;
use std::path::PathBuf;
mod stopwatch;
use clap::Parser;
use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::terminal::{Clear, ClearType};
use std::io::stdout;
use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use stopwatch::Args;
use tokio::{fs, signal};
async fn main_loop(current_stopwatch: Arc<stopwatch::Stopwatch>) {
    let mut stdout = stdout();
    let _ = stdout.execute(cursor::Hide);
    loop {
        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
        stdout.execute(cursor::MoveToColumn(0)).unwrap();
        let total_secs = current_stopwatch.get_time().as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        print!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let _ = stdout.flush();
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

async fn cleanup(current_stopwatch: Arc<stopwatch::Stopwatch>, path_to_write_to: Option<PathBuf>) {
    let _ = stdout().execute(cursor::Show);
    if let Some(path) = path_to_write_to {
        let current_time_secs = current_stopwatch.get_time().as_secs();
        let parsed_time: humantime::Duration = Duration::from_secs(current_time_secs).into();
        fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create path");
        let err = fs::write(path, parsed_time.to_string()).await;
        if err.is_err() {
            info!("Failed to log time")
        }
    }
}

fn setup_tracing() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn get_file_name() -> String {
    let filename_suffix = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    format!("stopwatch_{filename_suffix}.log")
}

#[tokio::main]
async fn main() {
    let given_config = tokio::task::spawn_blocking(config::get_config)
        .await
        .unwrap();
    let args = Args::parse();
    let time = args
        .time
        .map(|t| {
            t.parse::<humantime::Duration>()
                .expect("Invalid time format!")
        })
        .map(|t| -> Duration { t.into() })
        .unwrap_or(Duration::from_secs(0));
    setup_tracing();
    let filename_to_log = given_config
        .path_to_log_time
        .map(|p| p.join(get_file_name()));
    let current_stopwatch = Arc::new(stopwatch::Stopwatch::start_from(Instant::now() - time));
    tokio::spawn(main_loop(current_stopwatch.clone()));
    signal::ctrl_c().await.unwrap();
    cleanup(current_stopwatch.clone(), filename_to_log).await;
}
