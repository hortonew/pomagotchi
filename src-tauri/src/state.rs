use serde::{Deserialize, Serialize};

/// Represents the evolving creature the user nurtures.
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

impl CreatureState {
    /// Apply experience points and perform any necessary level ups / stage evolution.
    pub fn gain_experience(&mut self, points: u32) {
        self.xp += points;
        while self.xp >= self.xp_needed {
            self.level += 1;
            self.xp -= self.xp_needed;
            self.xp_needed = (self.xp_needed as f32 * 1.5) as u32;
        }
        self.stage = match self.level {
            1 => "egg".to_string(),
            2 => "baby".to_string(),
            3 => "teen".to_string(),
            _ => "adult".to_string(),
        };
    }
}

/// Tracks the current timer state for a Pomodoro session.
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

/// Aggregated meta progress stats for the user.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameProgress {
    pub total_pomodoros_completed: u32,
    pub total_xp_earned: u32,
    pub total_time_studied_seconds: u32,
    pub sessions_this_week: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub last_session_date: Option<String>, // ISO 8601 date string (YYYY-MM-DD)
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

/// All persisted game state.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creature_levels_up_and_updates_stage() {
        let mut c = CreatureState::default();
        // Give enough XP to level several times
        c.gain_experience(1000);
        assert!(c.level >= 2); // Should have leveled at least once
        assert!(matches!(c.stage.as_str(), "baby" | "teen" | "adult"));
    }
}
