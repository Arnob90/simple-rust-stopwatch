use clap::Parser;
use std::time;
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
#[derive(Parser, Debug)]
#[command(author, version, about = "A simple CLI stopwatch that saves your progress.", long_about = None)]
pub struct Args {
    /// Optional starting time (e.g., "1h 30m 10s").
    /// If provided, the stopwatch will count up from this duration.
    /// Uses humantime format
    #[arg(short, long)]
    pub time: Option<String>,
}
