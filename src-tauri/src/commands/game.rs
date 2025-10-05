use tauri::State;

use crate::app_state::SharedAppState as AppState;
use crate::state::GameState;

#[tauri::command]
pub async fn get_full_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.clone())
}

#[tauri::command]
pub async fn reset_game_data(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut game_state = state.game_state.lock().await;
        *game_state = GameState::default();
    }
    state.save_to_disk().await?;
    Ok(())
}

#[tauri::command]
pub async fn save_full_game_state(state: State<'_, AppState>, game_state: GameState) -> Result<(), String> {
    {
        let mut app_state = state.game_state.lock().await;
        *app_state = game_state;
    }
    state.save_to_disk().await?;
    Ok(())
}
