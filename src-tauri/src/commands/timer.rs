use tauri::State;

use crate::app_state::SharedAppState as AppState;
use crate::state::TimerState;

#[tauri::command]
pub async fn get_timer_state(state: State<'_, AppState>) -> Result<TimerState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.timer.clone())
}

#[tauri::command]
pub async fn update_timer_state(
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
    if !is_running {
        state.save_to_disk().await?;
    }
    Ok(())
}
