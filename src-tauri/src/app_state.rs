use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

use crate::state::GameState;

/// Shared application state managed by Tauri.
#[derive(Clone)]
pub struct AppState {
    pub game_state: Arc<Mutex<GameState>>,
    pub data_dir: PathBuf,
}

impl AppState {
    fn get_save_path(&self) -> PathBuf {
        self.data_dir.join("pomagotchi_save.json")
    }

    pub(crate) async fn save_to_disk(&self) -> Result<(), String> {
        let game_state = self.game_state.lock().await;
        let json = serde_json::to_string_pretty(&*game_state).map_err(|e| e.to_string())?;

        if let Some(parent) = self.get_save_path().parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }

        fs::write(self.get_save_path(), json).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub(crate) async fn load_from_disk(&self) -> Result<(), String> {
        let save_path = self.get_save_path();
        if !save_path.exists() {
            return Ok(()); // Nothing to load
        }
        let json = fs::read_to_string(save_path).await.map_err(|e| e.to_string())?;
        let loaded_state: GameState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let mut game_state = self.game_state.lock().await;
        *game_state = loaded_state;
        Ok(())
    }
}

pub(crate) fn is_consecutive_day(last_date: &str, current_date: &str) -> bool {
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

pub use AppState as SharedAppState; // Exported alias if needed externally.
