use std::thread;
use std::time::{Duration, Instant};
use std::env;
use std::process::Command;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    hooks: Hooks,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Hooks {
    work_start: Option<String>,
    work_end: Option<String>,
    break_start: Option<String>,
    break_end: Option<String>,
}

#[derive(Debug, Clone)]
enum TimerState {
    Work,
    Break,
    Idle,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args[1] != "start" {
        println!("Usage: pomo start");
        return;
    }

    let config = Arc::new(load_config());
    let timer_state = Arc::new(Mutex::new(TimerState::Idle));
    
    // Setup Ctrl-C handler
    let config_clone = Arc::clone(&config);
    let state_clone = Arc::clone(&timer_state);
    ctrlc::set_handler(move || {
        let state = state_clone.lock().unwrap();
        println!("\n🛑 Interrupted!");
        match *state {
            TimerState::Work => {
                execute_hook(&config_clone.hooks.work_end);
            },
            TimerState::Break => {
                execute_hook(&config_clone.hooks.break_end);
            },
            TimerState::Idle => {
                println!("No hook to execute (idle state)");
            },
        }
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    run_pomodoro(config, timer_state);
}

fn run_pomodoro(config: Arc<Config>, timer_state: Arc<Mutex<TimerState>>) {
    // 25-minute work timer
    println!("🍅 Starting 25-minute Pomodoro work session...");
    *timer_state.lock().unwrap() = TimerState::Work;
    execute_hook(&config.hooks.work_start);
    run_timer(25 * 60);
    execute_hook(&config.hooks.work_end);
    system_beep();
    println!("🍅 Work session complete! Time for a break.");
    
    // 5-minute break timer
    println!("☕ Starting 5-minute break...");
    *timer_state.lock().unwrap() = TimerState::Break;
    execute_hook(&config.hooks.break_start);
    run_timer(5 * 60);
    execute_hook(&config.hooks.break_end);
    system_beep();
    *timer_state.lock().unwrap() = TimerState::Idle;
    println!("☕ Break complete! Ready for another session?");
}

fn run_timer(duration_seconds: u64) {
    let start_time = Instant::now();
    let total_duration = Duration::from_secs(duration_seconds);
    
    while start_time.elapsed() < total_duration {
        let remaining = total_duration - start_time.elapsed();
        let minutes = remaining.as_secs() / 60;
        let seconds = remaining.as_secs() % 60;
        
        print!("\r⏱️  {:02}:{:02} remaining", minutes, seconds);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        thread::sleep(Duration::from_millis(1000));
    }
    
    println!("\r⏱️  00:00 - Time's up!");
}

fn load_config() -> Config {
    let config_path = get_config_path();
    
    if config_path.exists() {
        let config_content = fs::read_to_string(&config_path).unwrap_or_else(|_| {
            eprintln!("Warning: Could not read config file, using defaults");
            create_default_config(&config_path)
        });
        
        serde_json::from_str(&config_content).unwrap_or_else(|_| {
            eprintln!("Warning: Invalid config format, using defaults");
            let default_config = Config {
                hooks: Hooks {
                    work_start: None,
                    work_end: None,
                    break_start: None,
                    break_end: None,
                }
            };
            let _ = fs::write(&config_path, serde_json::to_string_pretty(&default_config).unwrap());
            default_config
        })
    } else {
        let default_config = Config {
            hooks: Hooks {
                work_start: None,
                work_end: None,
                break_start: None,
                break_end: None,
            }
        };
        
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        let config_json = create_default_config(&config_path);
        serde_json::from_str(&config_json).unwrap_or(default_config)
    }
}

fn get_config_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".config").join("pomo").join("config.json")
    } else {
        PathBuf::from("pomo-config.json")
    }
}

fn create_default_config(config_path: &PathBuf) -> String {
    let default_config = Config {
        hooks: Hooks {
            work_start: Some("# afplay ~/music/focus.mp3 &".to_string()),
            work_end: Some("# pkill afplay".to_string()),
            break_start: Some("# afplay ~/music/break.mp3 &".to_string()),
            break_end: Some("# pkill afplay".to_string()),
        }
    };
    
    let config_json = serde_json::to_string_pretty(&default_config).unwrap();
    let _ = fs::write(config_path, &config_json);
    config_json
}

fn execute_hook(hook: &Option<String>) {
    if let Some(command) = hook {
        if command.trim().starts_with('#') {
            println!("Hook is commented out: {}", command);
        } else if command.trim().is_empty() {
            println!("Hook is empty");
        } else {
            let _ = Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn();
        }
    } else {
        println!("No hook configured");
    }
}

fn system_beep() {
    if cfg!(target_os = "macos") {
        let _ = Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .spawn()
            .and_then(|mut child| child.wait());
    } else if cfg!(target_os = "linux") {
        let _ = Command::new("paplay")
            .arg("/usr/share/sounds/alsa/Front_Left.wav")
            .spawn()
            .and_then(|mut child| child.wait())
            .or_else(|_| Command::new("aplay")
                .arg("/usr/share/sounds/alsa/Front_Left.wav")
                .spawn()
                .and_then(|mut child| child.wait()));
    }
    
    // Fallback: ASCII bell character
    print!("\x07");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}
