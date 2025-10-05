use tauri::State;

use crate::app_state::SharedAppState as AppState;
use crate::state::CreatureState;

#[tauri::command]
pub async fn get_creature_state(state: State<'_, AppState>) -> Result<CreatureState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.creature.clone())
}

#[tauri::command]
pub async fn save_creature_state(state: State<'_, AppState>, level: u32, xp: u32, xp_needed: u32, stage: String) -> Result<(), String> {
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
pub async fn load_creature_state(state: State<'_, AppState>) -> Result<CreatureState, String> {
    let game_state = state.game_state.lock().await;
    Ok(game_state.creature.clone())
}

#[tauri::command]
pub async fn add_experience(state: State<'_, AppState>, points: u32) -> Result<CreatureState, String> {
    {
        let mut game_state = state.game_state.lock().await;
        game_state.progress.total_xp_earned += points;
        game_state.creature.gain_experience(points);
    }
    state.save_to_disk().await?;
    let game_state = state.game_state.lock().await;
    Ok(game_state.creature.clone())
}
