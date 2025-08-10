/// Configuration module for Pomodoro settings
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub work_duration: Duration,
    pub break_duration: Duration,
    pub cycles: usize,
    pub beep_sound: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            work_duration: Duration::from_secs(1500), // 25 minutes
            break_duration: Duration::from_secs(300), // 5 minutes
            cycles: 4,
            beep_sound: "\x07".to_string(), // ASCII bell character
        }
    }
}
