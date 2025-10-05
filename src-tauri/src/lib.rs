use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatureState {
    pub level: u32,
    pub xp: u32,
    pub xp_needed: u32,
    pub stage: String,
}

impl Default for CreatureState {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_needed: 100,
            stage: "egg".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimerState {
    pub minutes: u32,
    pub seconds: u32,
    pub is_running: bool,
    pub is_paused: bool,
    pub initial_total_seconds: u32,
    pub last_selected_minutes: u32,
    pub last_selected_seconds: u32,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            minutes: 25,
            seconds: 0,
            is_running: false,
            is_paused: false,
            initial_total_seconds: 1500,
            last_selected_minutes: 25,
            last_selected_seconds: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameProgress {
    pub total_pomodoros_completed: u32,
    pub total_xp_earned: u32,
    pub total_time_studied_seconds: u32,
    pub sessions_this_week: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub last_session_date: Option<String>, // ISO 8601 format
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            total_pomodoros_completed: 0,
            total_xp_earned: 0,
            total_time_studied_seconds: 0,
            sessions_this_week: 0,
            current_streak: 0,
            best_streak: 0,
            last_session_date: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub creature: CreatureState,
    pub timer: TimerState,
    pub progress: GameProgress,
    pub version: String,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            creature: CreatureState::default(),
            timer: TimerState::default(),
            progress: GameProgress::default(),
            version: "1.0.0".to_string(),
        }
    }
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub game_state: Arc<Mutex<GameState>>,
    pub data_dir: PathBuf,
}

impl AppState {
    fn get_save_path(&self) -> PathBuf {
        self.data_dir.join("pomagotchi_save.json")
    }

    async fn save_to_disk(&self) -> Result<(), String> {
        let game_state = self.game_state.lock().await;
        let json = serde_json::to_string_pretty(&*game_state).map_err(|e| e.to_string())?;

        // Ensure the data directory exists
        if let Some(parent) = self.get_save_path().parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }

        fs::write(self.get_save_path(), json).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn load_from_disk(&self) -> Result<(), String> {
        let save_path = self.get_save_path();

        if !save_path.exists() {
            return Ok(()); // No save file exists, use defaults
        }

        let json = fs::read_to_string(save_path).await.map_err(|e| e.to_string())?;
        let loaded_state: GameState = serde_json::from_str(&json).map_err(|e| e.to_string())?;

        let mut game_state = self.game_state.lock().await;
        *game_state = loaded_state;

        Ok(())
    }
}

#[tauri::command]
async fn get_creature_state(state: State<'_, AppState>) -> Result<CreatureState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.creature.clone())
}

#[tauri::command]
async fn save_creature_state(state: State<'_, AppState>, level: u32, xp: u32, xp_needed: u32, stage: String) -> Result<(), String> {
    {
        let mut game_state = state.game_state.lock().await;
        game_state.creature.level = level;
        game_state.creature.xp = xp;
        game_state.creature.xp_needed = xp_needed;
        game_state.creature.stage = stage;
    }
    state.save_to_disk().await?;
    Ok(())
}

#[tauri::command]
async fn load_creature_state(state: State<'_, AppState>) -> Result<CreatureState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.creature.clone())
}

#[tauri::command]
async fn add_experience(state: State<'_, AppState>, points: u32) -> Result<CreatureState, String> {
    {
        let mut game_state = state.game_state.lock().await;
        game_state.creature.xp += points;
        game_state.progress.total_xp_earned += points;

        // Check for evolution
        while game_state.creature.xp >= game_state.creature.xp_needed {
            game_state.creature.level += 1;
            game_state.creature.xp -= game_state.creature.xp_needed;
            game_state.creature.xp_needed = (game_state.creature.xp_needed as f32 * 1.5) as u32;

            // Update stage based on level
            game_state.creature.stage = match game_state.creature.level {
                1 => "egg".to_string(),
                2 => "baby".to_string(),
                3 => "teen".to_string(),
                _ => "adult".to_string(),
            };
        }
    }

    state.save_to_disk().await?;
    let game_state = state.game_state.lock().await;
    Ok(game_state.creature.clone())
}

#[tauri::command]
async fn complete_pomodoro(state: State<'_, AppState>, duration_seconds: u32, xp_gained: u32) -> Result<GameProgress, String> {
    {
        let mut game_state = state.game_state.lock().await;

        // Update progress tracking
        game_state.progress.total_pomodoros_completed += 1;
        game_state.progress.total_xp_earned += xp_gained;
        game_state.progress.total_time_studied_seconds += duration_seconds;

        // Update streak tracking
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if let Some(last_date) = &game_state.progress.last_session_date {
            if last_date == &today {
                // Same day, no streak change
            } else if is_consecutive_day(last_date, &today) {
                game_state.progress.current_streak += 1;
                game_state.progress.best_streak = game_state.progress.best_streak.max(game_state.progress.current_streak);
            } else {
                // Streak broken
                game_state.progress.current_streak = 1;
            }
        } else {
            // First session ever
            game_state.progress.current_streak = 1;
            game_state.progress.best_streak = 1;
        }

        game_state.progress.last_session_date = Some(today);

        // Add XP to creature
        game_state.creature.xp += xp_gained;

        // Check for evolution
        while game_state.creature.xp >= game_state.creature.xp_needed {
            game_state.creature.level += 1;
            game_state.creature.xp -= game_state.creature.xp_needed;
            game_state.creature.xp_needed = (game_state.creature.xp_needed as f32 * 1.5) as u32;

            // Update stage based on level
            game_state.creature.stage = match game_state.creature.level {
                1 => "egg".to_string(),
                2 => "baby".to_string(),
                3 => "teen".to_string(),
                _ => "adult".to_string(),
            };
        }
    }

    state.save_to_disk().await?;
    let game_state = state.game_state.lock().await;
    Ok(game_state.progress.clone())
}

#[tauri::command]
async fn get_timer_state(state: State<'_, AppState>) -> Result<TimerState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.timer.clone())
}

#[tauri::command]
async fn update_timer_state(
    state: State<'_, AppState>,
    minutes: u32,
    seconds: u32,
    is_running: bool,
    is_paused: bool,
    initial_total_seconds: u32,
    last_selected_minutes: u32,
    last_selected_seconds: u32,
) -> Result<(), String> {
    {
        let mut game_state = state.game_state.lock().await;
        game_state.timer.minutes = minutes;
        game_state.timer.seconds = seconds;
        game_state.timer.is_running = is_running;
        game_state.timer.is_paused = is_paused;
        game_state.timer.initial_total_seconds = initial_total_seconds;
        game_state.timer.last_selected_minutes = last_selected_minutes;
        game_state.timer.last_selected_seconds = last_selected_seconds;
    }

    // Only save to disk if timer is not running (to avoid frequent saves)
    if !is_running {
        state.save_to_disk().await?;
    }
    Ok(())
}

#[tauri::command]
async fn get_game_progress(state: State<'_, AppState>) -> Result<GameProgress, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.progress.clone())
}

#[tauri::command]
async fn get_full_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.clone())
}

#[tauri::command]
async fn reset_game_data(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut game_state = state.game_state.lock().await;
        *game_state = GameState::default();
    }

    state.save_to_disk().await?;
    Ok(())
}

#[tauri::command]
async fn save_full_game_state(state: State<'_, AppState>, game_state: GameState) -> Result<(), String> {
    {
        let mut app_state = state.game_state.lock().await;
        *app_state = game_state;
    }

    state.save_to_disk().await?;
    Ok(())
}

// Helper function to check if two dates are consecutive days
fn is_consecutive_day(last_date: &str, current_date: &str) -> bool {
    use chrono::{Duration, NaiveDate};

    if let (Ok(last), Ok(current)) = (
        NaiveDate::parse_from_str(last_date, "%Y-%m-%d"),
        NaiveDate::parse_from_str(current_date, "%Y-%m-%d"),
    ) {
        current - last == Duration::days(1)
    } else {
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_creature_state,
            save_creature_state,
            load_creature_state,
            add_experience,
            complete_pomodoro,
            get_timer_state,
            update_timer_state,
            get_game_progress,
            get_full_game_state,
            reset_game_data,
            save_full_game_state
        ])
        .setup(|app| {
            // Get the app data directory
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data directory: {}", e))?;

            let app_state = AppState {
                game_state: Arc::new(Mutex::new(GameState::default())),
                data_dir,
            };

            // Load saved state on startup
            let state = app_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = state.load_from_disk().await {
                    eprintln!("Failed to load saved state: {}", e);
                }
            });

            app.manage(app_state);

            if cfg!(debug_assertions) {
                app.handle()
                    .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
