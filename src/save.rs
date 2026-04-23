use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::{UpgradeKind, upgrade_index};

/// Persistent player progress saved to disk.
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct SaveData {
    /// Best star rating per level (0 = not beaten). Index 0 = level 1.
    pub level_stars: Vec<u8>,
    /// Meta-currency earned from completing levels.
    pub aether_gems: u32,
    /// Upgrade levels for each meta-progression upgrade (0 = not purchased). Index maps to UpgradeKind ordinal.
    pub upgrade_levels: Vec<u8>,
    /// Whether the first-play tutorial has been completed (or skipped).
    #[serde(default)]
    pub tutorial_completed: bool,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            level_stars: vec![0; 10],
            aether_gems: 0,
            upgrade_levels: vec![0; 5],
            tutorial_completed: false,
        }
    }
}

/// Returns the save file path.
fn save_file_path() -> String {
    #[cfg(target_os = "android")]
    {
        // Android internal storage — ndk_glue provides the path
        // Fallback to a relative path that maps to the app's internal dir
        "/data/data/com.agesofaether/files/save.json".to_string()
    }
    #[cfg(not(target_os = "android"))]
    {
        "save.json".to_string()
    }
}

/// Loads save data from disk on startup. Inserts default if file missing or corrupt.
pub fn load_save_on_startup(mut commands: Commands) {
    let path = save_file_path();
    let data = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<SaveData>(&s).ok())
        .unwrap_or_default();
    // Ensure vectors are long enough (handles save from older version with fewer levels)
    let mut data = data;
    data.level_stars.resize(10, 0);
    data.upgrade_levels.resize(5, 0);
    commands.insert_resource(data);
}

impl SaveData {
    /// Get the level of a specific upgrade (0 if not purchased).
    pub fn upgrade_level(&self, kind: UpgradeKind) -> u8 {
        let idx = upgrade_index(kind);
        if idx < self.upgrade_levels.len() { self.upgrade_levels[idx] } else { 0 }
    }

    /// Tower damage multiplier from ElementalFury upgrade.
    pub fn tower_damage_mult(&self) -> f32 {
        1.0 + 0.05 * self.upgrade_level(UpgradeKind::ElementalFury) as f32
    }

    /// Tower range multiplier from FarSight upgrade.
    pub fn tower_range_mult(&self) -> f32 {
        1.0 + 0.05 * self.upgrade_level(UpgradeKind::FarSight) as f32
    }

    /// Sell refund rate including SalvageExpert bonus.
    pub fn sell_refund_rate(&self) -> f32 {
        crate::data::SELL_REFUND_RATE + 0.05 * self.upgrade_level(UpgradeKind::SalvageExpert) as f32
    }

    /// Hero ability cooldown multiplier from TacticalMastery upgrade.
    pub fn cooldown_mult(&self) -> f32 {
        1.0 - 0.05 * self.upgrade_level(UpgradeKind::TacticalMastery) as f32
    }
}

/// Writes the save data to disk. Call after updating any persistent field.
pub fn write_save(save: &SaveData) {
    let path = save_file_path();
    if let Ok(json) = serde_json::to_string_pretty(save) {
        if let Err(e) = std::fs::write(&path, json) {
            warn!("Failed to write save file: {}", e);
        }
    }
}

/// Saves progress when entering GameOver state (only on victory).
pub fn save_on_level_complete(
    outcome: Res<crate::states::GameOutcome>,
    current_level: Res<crate::resources::CurrentLevel>,
    mut save: ResMut<SaveData>,
) {
    if !outcome.victory || outcome.stars == 0 {
        return;
    }

    let idx = (current_level.0 as usize).saturating_sub(1);
    if idx >= save.level_stars.len() {
        return;
    }

    // Only update if new star count is better
    let old_stars = save.level_stars[idx];
    if outcome.stars > old_stars {
        let new_stars = outcome.stars - old_stars;
        save.level_stars[idx] = outcome.stars;
        // Award gems: 10 per star earned (only for new stars)
        let gems = new_stars as u32 * 10;
        save.aether_gems += gems;
        info!("Level {} complete! {} stars (+{} gems)", current_level.0, outcome.stars, gems);
    }

    // Write to disk
    let path = save_file_path();
    if let Ok(json) = serde_json::to_string_pretty(&*save) {
        if let Err(e) = std::fs::write(&path, json) {
            warn!("Failed to write save file: {}", e);
        }
    }
}
