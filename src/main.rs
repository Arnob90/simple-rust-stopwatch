use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
mod stopwatch;
use clap::Parser;
use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::terminal::{Clear, ClearType};
use std::io::stdout;
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

async fn cleanup(current_stopwatch: Arc<stopwatch::Stopwatch>) {
    let _ = stdout().execute(cursor::Show);
    let path = std::env::current_exe().unwrap();
    let directory = path.parent().unwrap();
    let times_path = directory.join("Times/");
    fs::create_dir_all(&times_path)
        .await
        .expect("Could not create path to write file");
    let mut i = 1;
    loop {
        let time_path = times_path.join(format!("time_saved_{i}.txt"));
        if !time_path.exists() {
            let current_time = current_stopwatch.get_time();
            let current_time_approx = Duration::from_secs(current_time.as_secs());
            let parsed_time: humantime::Duration = current_time_approx.into();
            fs::write(time_path, parsed_time.to_string())
                .await
                .expect("Could not write file");
            break;
        }
        i += 1;
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let time = args
        .time
        .map(|t| {
            t.parse::<humantime::Duration>()
                .expect("Invalid time format!")
        })
        .map(|t| -> Duration { t.into() })
        .unwrap_or(Duration::from_secs(0));
    let current_stopwatch = Arc::new(stopwatch::Stopwatch::start_from(Instant::now() - time));
    tokio::spawn(main_loop(current_stopwatch.clone()));
    signal::ctrl_c().await.unwrap();
    cleanup(current_stopwatch.clone()).await;
}
