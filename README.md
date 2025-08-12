# Pomo 🍅

A simple command-line Pomodoro timer written in Rust.

## Features

- 25-minute work sessions followed by 5-minute breaks
- Configurable hooks for work/break start/end events
- Audio notifications using system sounds
- Graceful interruption handling with Ctrl+C
- Cross-platform support (macOS, Linux)

## Installation

```bash
cargo build --release
```

## Usage

Start a Pomodoro session:
```bash
cargo run start
```

Or with the built binary:
```bash
./target/release/pomo start
```

### Custom Duration

Set custom work session duration:
```bash
cargo run start --duration 25m    # 25 minutes (default)
cargo run start --duration 30s    # 30 seconds
cargo run start --duration 1m30s  # 1 minute 30 seconds
cargo run start --duration 5      # 5 minutes (backward compatible)
```

### Options

- `--duration <time>` - Set work session duration (formats: 25m, 30s, 1m30s, or plain number for minutes)
- `--no-music` - Disable audio hooks and system beeps

## Configuration

Pomo creates a configuration file at `~/.config/pomo/config.json` with customizable hooks:

```json
{
  "hooks": {
    "work_start": "# afplay ~/music/focus.mp3 &",
    "work_end": "# pkill afplay",
    "break_start": "# afplay ~/music/break.mp3 &",
    "break_end": "# pkill afplay"
  }
}
```

- Hooks starting with `#` are treated as comments and won't execute
- Remove the `#` to enable a hook
- Customize commands for your preferred audio player or other actions

## How It Works

1. **Work Session**: 25-minute focused work period
2. **Break**: 5-minute rest period
3. **Audio Cues**: System beep notifications at session transitions
4. **Hooks**: Execute custom commands at session start/end
5. **Interruption Handling**: Clean exit with Ctrl+C runs appropriate end hooks

## Dependencies

- `serde` - Configuration serialization
- `ctrlc` - Signal handling for graceful interruption

## Platform Notes

- **macOS**: Uses `afplay` for system sounds
- **Linux**: Uses `paplay` or `aplay` for system sounds
- **Fallback**: ASCII bell character for audio notification