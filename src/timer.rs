/// Timer module for handling Pomodoro sessions
use std::thread;
use std::time::Duration;

pub fn start_timer(duration: Duration) {
    thread::sleep(duration);
    println!("Time's up! \x07"); // \x07 is ASCII bell character
}

pub fn duration_from_minutes(minutes: u64) -> Duration {
    Duration::from_secs(minutes * 60)
}
