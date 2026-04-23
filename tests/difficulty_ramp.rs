//! Difficulty-ramp analysis.
//!
//! For every level, walks each wave and reports:
//!   - total wave HP (scaled)
//!   - wave gold reward
//!   - cumulative gold available at the START of the wave
//!     (starting_gold + rewards from all prior waves)
//!   - HP-delta factor vs previous wave (HP[N] / HP[N-1])
//!   - gold-surplus ratio (cumulative gold / wave HP)
//!
//! Run with:  cargo test --test difficulty_ramp -- --nocapture
//!
//! A "cliff" shows up as a large HP-delta factor (>> 1.5x) or a sudden
//! drop in the gold-surplus ratio. A "sandbag" shows up as a very high
//! gold-surplus ratio early on — the player accumulates gold with
//! nothing threatening to spend on.

use ages_of_aether::data::*;

/// Compute the total effective HP of a single wave given the level's HP scale.
fn wave_total_hp(wave: &WaveDefinition, wave_index: u32, hp_scale: f32) -> f32 {
    let scale = 1.0 + wave_index as f32 * hp_scale;
    wave.groups.iter().fold(0.0, |acc, g| {
        acc + enemy_stats(g.enemy_type).hp * g.count as f32 * scale
    })
}

/// Gold reward for clearing a wave (sum of per-enemy rewards, no early-call bonus).
fn wave_gold_reward(wave: &WaveDefinition) -> u32 {
    wave.groups.iter().fold(0u32, |acc, g| {
        acc + enemy_stats(g.enemy_type).gold_reward * g.count
    })
}

#[test]
fn print_difficulty_profile() {
    println!();
    println!("=== Difficulty profile per level ===");
    println!("(HPx = wave HP / previous wave HP;  Surplus = gold-at-start / wave HP)");
    println!();

    for level in 1..=10u32 {
        let cfg = level_start_config(level);
        let waves = level_waves(level);
        let info = crate::level_info(level);
        println!(
            "--- Level {} ({}) start_gold={} hp_scale={:.2} ---",
            level, info.name, cfg.starting_gold, cfg.wave_hp_scale,
        );
        println!(
            "  {:<4} {:>8} {:>6} {:>8} {:>6} {:>7}",
            "wave", "hp", "gold", "cumGold", "HPx", "surplus"
        );

        let mut cum_gold = cfg.starting_gold as f32;
        let mut prev_hp = 0.0f32;
        for (i, wave) in waves.iter().enumerate() {
            let hp = wave_total_hp(wave, i as u32, cfg.wave_hp_scale);
            let reward = wave_gold_reward(wave);
            let hpx = if prev_hp > 0.0 { hp / prev_hp } else { 1.0 };
            let surplus = cum_gold / hp;
            println!(
                "  W{:<3} {:>8.0} {:>6} {:>8.0} {:>6.2} {:>7.2}",
                i + 1,
                hp,
                reward,
                cum_gold,
                hpx,
                surplus,
            );
            cum_gold += reward as f32;
            prev_hp = hp;
        }
        println!();
    }
}

// Re-export of level_info from ages_of_aether (it's pub there, convenience alias).
mod level_info_shim {
    pub use ages_of_aether::data::level_info;
}
use level_info_shim::level_info;
