mod app_state;
mod commands;
mod state;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub use app_state::SharedAppState as AppState;
pub use commands::*;
pub use state::*;

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

            let app_state = crate::app_state::AppState {
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
