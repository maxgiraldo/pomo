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
        println!("Usage: pomo start [--duration <time>] [--no-music]");
        println!("  <time> format: 25m, 30s, 1m30s");
        return;
    }
    
    let no_music = args.contains(&"--no-music".to_string());
    
    // Parse duration flag
    let mut duration_seconds = 25 * 60; // default 25 minutes in seconds
    if let Some(duration_index) = args.iter().position(|x| x == "--duration") {
        if duration_index + 1 < args.len() {
            match parse_duration(&args[duration_index + 1]) {
                Ok(seconds) => {
                    if seconds > 0 {
                        duration_seconds = seconds;
                    } else {
                        println!("Error: Duration must be greater than 0");
                        return;
                    }
                }
                Err(err) => {
                    println!("Error: {}", err);
                    return;
                }
            }
        } else {
            println!("Error: --duration flag requires a value");
            return;
        }
    }

    let config = Arc::new(load_config());
    let timer_state = Arc::new(Mutex::new(TimerState::Idle));
    
    // Setup Ctrl-C handler
    let config_clone = Arc::clone(&config);
    let state_clone = Arc::clone(&timer_state);
    let no_music_clone = no_music;
    ctrlc::set_handler(move || {
        let state = state_clone.lock().unwrap();
        println!("\n🛑 Interrupted!");
        match *state {
            TimerState::Work => {
                if !no_music_clone {
                    execute_hook(&config_clone.hooks.work_end);
                }
            },
            TimerState::Break => {
                if !no_music_clone {
                    execute_hook(&config_clone.hooks.break_end);
                }
            },
            TimerState::Idle => {
                println!("No hook to execute (idle state)");
            },
        }
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    run_pomodoro(config, timer_state, no_music, duration_seconds);
}

fn run_pomodoro(config: Arc<Config>, timer_state: Arc<Mutex<TimerState>>, no_music: bool, duration_seconds: u64) {
    // Work timer
    let duration_display = format_duration(duration_seconds);
    println!("🍅 Starting {} Pomodoro work session...", duration_display);
    *timer_state.lock().unwrap() = TimerState::Work;
    if !no_music {
        execute_hook(&config.hooks.work_start);
    }
    run_timer(duration_seconds);
    if !no_music {
        execute_hook(&config.hooks.work_end);
        thread::sleep(Duration::from_millis(200)); // Allow time for hook to complete before system beep
    }
    if !no_music {
        system_beep();
    }
    println!("🍅 Work session complete! Time for a break.");
    
    // 5-minute break timer
    println!("☕ Starting 5-minute break...");
    *timer_state.lock().unwrap() = TimerState::Break;
    if !no_music {
        execute_hook(&config.hooks.break_start);
    }
    run_timer(5 * 60);
    if !no_music {
        execute_hook(&config.hooks.break_end);
        thread::sleep(Duration::from_millis(200)); // Allow time for hook to complete before system beep
    }
    if !no_music {
        system_beep();
    }
    *timer_state.lock().unwrap() = TimerState::Idle;
    println!("☕ Break complete! Ready for another session?");
}

fn run_timer(duration_seconds: u64) {
    let start_time = Instant::now();
    let total_duration = Duration::from_secs(duration_seconds);
    
    while start_time.elapsed() < total_duration {
        let elapsed = start_time.elapsed();
        let remaining = total_duration - elapsed;
        
        // Time formatting
        let minutes = remaining.as_secs() / 60;
        let seconds = remaining.as_secs() % 60;
        
        // Progress calculation
        let progress_ratio = elapsed.as_secs_f64() / total_duration.as_secs_f64();
        let percentage = (progress_ratio * 100.0) as u8;
        
        // Progress bar generation
        let bar_width = 20;
        let filled_blocks = (progress_ratio * bar_width as f64) as usize;
        let empty_blocks = bar_width - filled_blocks;
        
        let progress_bar = format!("{}{}",
            "█".repeat(filled_blocks),
            "░".repeat(empty_blocks)
        );
        
        // Combined display
        print!("\r⏱️  {:02}:{:02} remaining [{}] {}%", 
            minutes, seconds, progress_bar, percentage);
        
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        thread::sleep(Duration::from_millis(1000));
    }
    
    println!("\r⏱️  00:00 - Time's up! [{}] 100%", "█".repeat(20));
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

fn parse_duration(input: &str) -> Result<u64, String> {
    let input = input.trim().to_lowercase();
    
    // If it's just a number, treat as minutes for backward compatibility
    if let Ok(minutes) = input.parse::<u64>() {
        return Ok(minutes * 60);
    }
    
    let mut total_seconds = 0u64;
    let mut current_number = String::new();
    
    for ch in input.chars() {
        if ch.is_ascii_digit() {
            current_number.push(ch);
        } else if ch == 'm' || ch == 's' {
            if current_number.is_empty() {
                return Err("Invalid duration format. Use formats like: 25m, 30s, 1m30s".to_string());
            }
            
            let number: u64 = current_number.parse()
                .map_err(|_| "Invalid number in duration".to_string())?;
            
            match ch {
                'm' => total_seconds += number * 60,
                's' => total_seconds += number,
                _ => unreachable!(),
            }
            
            current_number.clear();
        } else if !ch.is_whitespace() {
            return Err("Invalid character in duration. Use formats like: 25m, 30s, 1m30s".to_string());
        }
    }
    
    if !current_number.is_empty() {
        return Err("Duration must end with 'm' (minutes) or 's' (seconds)".to_string());
    }
    
    if total_seconds == 0 {
        return Err("Duration must be greater than 0".to_string());
    }
    
    Ok(total_seconds)
}

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    
    if minutes > 0 && remaining_seconds > 0 {
        format!("{} minute{} {} second{}", 
            minutes, if minutes == 1 { "" } else { "s" },
            remaining_seconds, if remaining_seconds == 1 { "" } else { "s" })
    } else if minutes > 0 {
        format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" })
    } else {
        format!("{} second{}", remaining_seconds, if remaining_seconds == 1 { "" } else { "s" })
    }
}

fn system_beep() {
    let mut sound_played = false;
    
    if cfg!(target_os = "macos") {
        // Try different macOS system sounds
        let sounds = [
            "/System/Library/Sounds/Glass.aiff",
            "/System/Library/Sounds/Ping.aiff",
            "/System/Library/Sounds/Pop.aiff",
            "/System/Library/Sounds/Purr.aiff"
        ];
        
        for sound_path in &sounds {
            if let Ok(mut child) = Command::new("afplay")
                .arg(sound_path)
                .spawn() {
                if child.wait().is_ok() {
                    sound_played = true;
                    break;
                }
            }
        }
        
        // Fallback to say command for macOS
        if !sound_played {
            if let Ok(mut child) = Command::new("say")
                .arg("Time up")
                .spawn() {
                let _ = child.wait();
                sound_played = true;
            }
        }
    } else if cfg!(target_os = "linux") {
        let sounds = [
            "/usr/share/sounds/alsa/Front_Left.wav",
            "/usr/share/sounds/sound-icons/bell.wav",
            "/usr/share/sounds/gnome/default/alerts/glass.ogg"
        ];
        
        for sound_path in &sounds {
            if let Ok(mut child) = Command::new("paplay")
                .arg(sound_path)
                .spawn() {
                if child.wait().is_ok() {
                    sound_played = true;
                    break;
                }
            }
        }
        
        if !sound_played {
            for sound_path in &sounds {
                if let Ok(mut child) = Command::new("aplay")
                    .arg(sound_path)
                    .spawn() {
                    if child.wait().is_ok() {
                        sound_played = true;
                        break;
                    }
                }
            }
        }
    }
    
    // Always print the bell character as fallback
    if !sound_played {
        print!("\x07");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}
