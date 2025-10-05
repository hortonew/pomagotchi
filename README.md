# Pomagotchi

A Pomodoro + Tamagotchi productivity application that gamifies focus time by letting you grow and evolve a virtual creature through completed Pomodoro sessions.

## Tech Stack

- **Backend**: Rust (Tauri 2.0)
- **Frontend**: Vanilla JavaScript, HTML, CSS (Tailwind CSS)
- **State Management**: Rust with async Tokio for persistent JSON storage
- **Notifications**: Tauri Plugin Notification
- **Targets**: Desktop (macOS, Windows, Linux) & Android

## Key Features

- Pomodoro timer with customizable preset durations
- Creature evolution system with level progression and XP
- Four evolution stages: Egg → Baby → Teen → Adult
- Persistent state management across sessions
- Streak tracking and progress statistics
- Session history with total Pomodoros, XP earned, and study time
- Desktop notifications for session completion
- Local JSON-based save system

## Development

### Prerequisites

- Rust (latest stable)
- [just](https://github.com/casey/just) command runner
- Platform-specific tools for Tauri (see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/))

### Quick Start

```bash
# List available commands
just

# Run in development mode
just dev

# Build for production
just run
```

### Android Development

```bash
# Initialize Android
just android-rebuild

# Run on Android emulator
just android

# Build and install debug APK to device
just debug
```

### iOS Development

Possible, but not yet configured.

## Project Structure

- `src/` - Frontend (HTML, CSS, JS)
- `src-tauri/src/` - Rust backend
  - `lib.rs` - Main library file with Tauri commands and state management
  - `main.rs` - Application entry point

## How It Works

The app combines Pomodoro productivity with creature care mechanics. When you complete a Pomodoro session, your creature gains experience points. As XP accumulates, the creature levels up and evolves through different life stages. The app tracks your progress, maintains streaks, and persists all data locally between sessions.
