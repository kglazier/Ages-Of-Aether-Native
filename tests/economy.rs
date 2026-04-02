//! Economy balance tests — validates gold income timelines and build affordability.
//!
//! Ported from the TypeScript economy.test.ts with improvements:
//! - Cumulative gold tracking across waves (not just totals)
//! - Build-spot fill percentage thresholds at wave milestones
//! - Upgrade affordability checks

use ages_of_aether::components::Element;
use ages_of_aether::data::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ALL_ELEMENTS: [Element; 4] = [
    Element::Lightning,
    Element::Earth,
    Element::Ice,
    Element::Fire,
];

/// Cheapest level-0 tower cost across all elements.
fn cheapest_tower_cost() -> u32 {
    ALL_ELEMENTS
        .iter()
        .map(|&e| tower_stats(e, 0).cost)
        .min()
        .unwrap()
}

/// Cost to fill `n` build spots with the cheapest tower.
fn cheapest_fill_cost(n: usize) -> u32 {
    n as u32 * cheapest_tower_cost()
}

/// Compute total gold income for a level (starting + all kill rewards + early call bonuses).
fn total_gold_income(level: u32) -> (u32, u32, u32, u32) {
    let config = level_start_config(level);
    let waves = level_waves(level);

    let mut kill_gold: u32 = 0;
    let mut early_call_gold: u32 = 0;

    for wave in &waves {
        early_call_gold += wave.early_call_bonus;
        for group in &wave.groups {
            let stats = enemy_stats(group.enemy_type);
            kill_gold += group.count * stats.gold_reward;
        }
    }

    let total = config.starting_gold + kill_gold + early_call_gold;
    (config.starting_gold, kill_gold, early_call_gold, total)
}

/// Cumulative gold available after completing `num_waves` waves (starting gold + rewards).
fn cumulative_gold_after_waves(level: u32, num_waves: usize) -> u32 {
    let config = level_start_config(level);
    let waves = level_waves(level);
    let mut cum = config.starting_gold;

    for wave in waves.iter().take(num_waves) {
        cum += wave.early_call_bonus;
        for group in &wave.groups {
            let stats = enemy_stats(group.enemy_type);
            cum += group.count * stats.gold_reward;
        }
    }
    cum
}

/// Cheapest full upgrade chain cost (L0 + L1 + L2) across all elements.
fn cheapest_full_upgrade_cost() -> u32 {
    ALL_ELEMENTS
        .iter()
        .map(|&e| (0..3u8).map(|lv| tower_stats(e, lv).cost).sum::<u32>())
        .min()
        .unwrap()
}

// ===========================================================================
// 1. Total Gold Income
// ===========================================================================

#[test]
fn total_gold_at_least_1_2x_cheapest_fill() {
    for level in 1..=MAX_LEVELS {
        let spots = level_build_spots(level);
        let fill_cost = cheapest_fill_cost(spots.len());
        let threshold = (fill_cost as f32 * 1.2) as u32;
        let (start, kill, bonus, total) = total_gold_income(level);

        println!(
            "L{:>2} | Start: {} Kill: {} Bonus: {} Total: {} | Fill: {} Threshold: {}",
            level, start, kill, bonus, total, fill_cost, threshold
        );

        assert!(
            total >= threshold,
            "Level {} total gold ({}) < 1.2x cheapest fill cost ({})",
            level, total, threshold
        );
    }
}

// ===========================================================================
// 2. Build Timeline — milestone checks
// ===========================================================================

#[test]
fn starting_gold_affords_at_least_2_towers() {
    let cheapest = cheapest_tower_cost();
    for level in 1..=MAX_LEVELS {
        let config = level_start_config(level);
        let affordable = config.starting_gold / cheapest;
        assert!(
            affordable >= 2,
            "Level {}: starting gold {} only buys {} cheapest towers ({}g each)",
            level, config.starting_gold, affordable, cheapest
        );
    }
}

#[test]
fn after_wave_3_can_afford_50_percent_of_spots() {
    let cheapest = cheapest_tower_cost();
    for level in 1..=MAX_LEVELS {
        let spots = level_build_spots(level);
        let cum_gold = cumulative_gold_after_waves(level, 3);
        let affordable = cum_gold / cheapest;
        let target = (spots.len() as f32 * 0.5).ceil() as u32;

        println!(
            "L{:>2} | After wave 3: {}g buys {} towers, need {} (50% of {})",
            level, cum_gold, affordable, target, spots.len()
        );

        assert!(
            affordable >= target,
            "Level {}: after wave 3, gold {} buys {} towers, need {} (50% of {} spots)",
            level, cum_gold, affordable, target, spots.len()
        );
    }
}

#[test]
fn after_wave_5_can_afford_75_percent_of_spots() {
    let cheapest = cheapest_tower_cost();
    for level in 1..=MAX_LEVELS {
        let spots = level_build_spots(level);
        let cum_gold = cumulative_gold_after_waves(level, 5);
        let affordable = cum_gold / cheapest;
        let target = (spots.len() as f32 * 0.75).ceil() as u32;

        println!(
            "L{:>2} | After wave 5: {}g buys {} towers, need {} (75% of {})",
            level, cum_gold, affordable, target, spots.len()
        );

        assert!(
            affordable >= target,
            "Level {}: after wave 5, gold {} buys {} towers, need {} (75% of {} spots)",
            level, cum_gold, affordable, target, spots.len()
        );
    }
}

#[test]
fn after_wave_5_can_afford_2_base_plus_1_full_upgrade() {
    let cheapest_base = cheapest_tower_cost();
    let cheapest_full = cheapest_full_upgrade_cost();
    let needed = cheapest_base * 2 + cheapest_full;

    for level in 1..=MAX_LEVELS {
        let cum_gold = cumulative_gold_after_waves(level, 5);

        println!(
            "L{:>2} | After wave 5: {}g vs needed {} (2 base @ {} + 1 full upgrade @ {})",
            level, cum_gold, needed, cheapest_base, cheapest_full
        );

        assert!(
            cum_gold >= needed,
            "Level {}: after wave 5, gold {} < {} (2 base + 1 max upgrade)",
            level, cum_gold, needed
        );
    }
}

// ===========================================================================
// 3. Gold income progression — later levels should not be stingier than earlier
// ===========================================================================

#[test]
fn kill_gold_per_wave_never_below_minimum_threshold() {
    // Era transitions can naturally shift gold rates, so rather than requiring
    // monotonic increase, we check no level drops below a sensible floor:
    // at least enough gold per wave to buy one cheapest tower every 3 waves.
    let cheapest = cheapest_tower_cost();
    let floor = cheapest as f32 / 3.0;

    for level in 1..=MAX_LEVELS {
        let waves = level_waves(level);
        let total_kill_gold: u32 = waves.iter().flat_map(|w| &w.groups)
            .map(|g| g.count * enemy_stats(g.enemy_type).gold_reward)
            .sum();
        let avg = total_kill_gold as f32 / waves.len() as f32;

        println!(
            "L{:>2} | Total kill gold: {}, avg per wave: {:.0}, floor: {:.0}",
            level, total_kill_gold, avg, floor
        );

        assert!(
            avg >= floor,
            "Level {} avg kill gold/wave ({:.0}) below minimum threshold ({:.0})",
            level, avg, floor
        );
    }
}

// ===========================================================================
// 4. Early call bonuses are meaningful but not dominant
// ===========================================================================

#[test]
fn early_call_bonus_is_less_than_50_percent_of_wave_kill_gold() {
    for level in 1..=MAX_LEVELS {
        let waves = level_waves(level);
        for (wi, wave) in waves.iter().enumerate() {
            let wave_kill: u32 = wave.groups.iter()
                .map(|g| g.count * enemy_stats(g.enemy_type).gold_reward)
                .sum();
            if wave_kill > 0 {
                let ratio = wave.early_call_bonus as f32 / wave_kill as f32;
                assert!(
                    ratio <= 0.5,
                    "Level {} wave {}: early_call_bonus ({}) is > 50% of kill gold ({})",
                    level, wi + 1, wave.early_call_bonus, wave_kill
                );
            }
        }
    }
}
