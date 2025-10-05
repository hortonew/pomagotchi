use tauri::State;

use crate::app_state::{is_consecutive_day, SharedAppState as AppState};
use crate::state::GameProgress;

#[tauri::command]
pub async fn complete_pomodoro(state: State<'_, AppState>, duration_seconds: u32, xp_gained: u32) -> Result<GameProgress, String> {
    {
        let mut game_state = state.game_state.lock().await;

        game_state.progress.total_pomodoros_completed += 1;
        game_state.progress.total_xp_earned += xp_gained;
        game_state.progress.total_time_studied_seconds += duration_seconds;

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if let Some(last_date) = &game_state.progress.last_session_date {
            if last_date == &today {
                // Same day, no streak change
            } else if is_consecutive_day(last_date, &today) {
                game_state.progress.current_streak += 1;
                game_state.progress.best_streak = game_state.progress.best_streak.max(game_state.progress.current_streak);
            } else {
                game_state.progress.current_streak = 1; // Streak reset
            }
        } else {
            game_state.progress.current_streak = 1;
            game_state.progress.best_streak = 1;
        }
        game_state.progress.last_session_date = Some(today);

        game_state.creature.gain_experience(xp_gained);
    }

    state.save_to_disk().await?;
    let game_state = state.game_state.lock().await;
    Ok(game_state.progress.clone())
}

#[tauri::command]
pub async fn get_game_progress(state: State<'_, AppState>) -> Result<GameProgress, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.progress.clone())
}
