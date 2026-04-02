//! Era-based enemy validation and time-to-kill matrix tests.
//!
//! Ported from the TypeScript enemies.test.ts with improvements:
//! - Era grouping derived from level wave data (same approach as JS)
//! - HP/reward ratio consistency per era
//! - Boss HP and reward scaling
//! - Per-era archetype coverage (high-armor, magic-resist, flying, healer)
//! - TTK matrix: no enemy immune to any element, TTK limits with focused fire

use ages_of_aether::components::Element;
use ages_of_aether::data::*;
use std::collections::{HashMap, HashSet};

const ALL_ELEMENTS: [Element; 4] = [
    Element::Lightning,
    Element::Earth,
    Element::Ice,
    Element::Fire,
];

const BOSS_TYPES: [EnemyType; 5] = [
    EnemyType::GiantWorm,
    EnemyType::TRex,
    EnemyType::WoollyRhino,
    EnemyType::Minotaur,
    EnemyType::Dragon,
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_boss(et: EnemyType) -> bool {
    BOSS_TYPES.contains(&et)
}

fn is_magic(element: Element) -> bool {
    matches!(element, Element::Lightning | Element::Ice)
}

fn effective_dps_for(element: Element, level: u8, armor: f32, mr: f32) -> f32 {
    let s = tower_stats(element, level);
    let raw = s.damage * s.attack_speed;
    if is_magic(element) {
        raw * (1.0 - mr)
    } else {
        raw * (1.0 - armor / (armor + 100.0))
    }
}

/// Build a map from era name -> set of enemy types that appear in that era's levels.
/// Derives era membership from level_info().era and level_waves().
fn build_era_enemies() -> (Vec<String>, HashMap<String, Vec<EnemyType>>) {
    let mut era_order: Vec<String> = Vec::new();
    let mut era_set: HashMap<String, HashSet<EnemyType>> = HashMap::new();

    for level in 1..=MAX_LEVELS {
        let info = level_info(level);
        let era = info.era.to_string();
        if !era_set.contains_key(&era) {
            era_order.push(era.clone());
            era_set.insert(era.clone(), HashSet::new());
        }
        let set = era_set.get_mut(&era).unwrap();
        let waves = level_waves(level);
        for wave in &waves {
            for group in &wave.groups {
                set.insert(group.enemy_type);
            }
        }
    }

    let era_enemies: HashMap<String, Vec<EnemyType>> = era_set
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();
    (era_order, era_enemies)
}

// ===========================================================================
// 1. HP/Reward Ratios — consistency within each era
// ===========================================================================

#[test]
fn hp_to_gold_ratios_consistent_within_era() {
    let (era_order, era_enemies) = build_era_enemies();

    for era in &era_order {
        let enemies = &era_enemies[era];
        let non_boss: Vec<EnemyType> = enemies.iter().copied().filter(|&et| !is_boss(et)).collect();
        if non_boss.len() < 2 {
            continue;
        }

        let ratios: Vec<(EnemyType, f32)> = non_boss
            .iter()
            .map(|&et| {
                let s = enemy_stats(et);
                (et, s.hp / s.gold_reward as f32)
            })
            .collect();
        let avg: f32 = ratios.iter().map(|(_, r)| r).sum::<f32>() / ratios.len() as f32;

        for &(et, ratio) in &ratios {
            let deviation = (ratio - avg).abs() / avg;
            assert!(
                deviation <= 1.5,
                "{} era: {:?} hp/gold ratio {:.1} deviates {:.0}% from avg {:.1}",
                era, et, ratio, deviation * 100.0, avg
            );
        }
    }
}

#[test]
fn boss_hp_is_1_5_to_30x_era_average() {
    let (era_order, era_enemies) = build_era_enemies();

    for era in &era_order {
        let enemies = &era_enemies[era];
        let non_boss: Vec<EnemyType> = enemies.iter().copied().filter(|&et| !is_boss(et)).collect();
        let bosses: Vec<EnemyType> = enemies.iter().copied().filter(|&et| is_boss(et)).collect();

        if non_boss.is_empty() || bosses.is_empty() {
            continue;
        }

        let avg_hp: f32 = non_boss.iter().map(|&et| enemy_stats(et).hp).sum::<f32>()
            / non_boss.len() as f32;

        for &boss in &bosses {
            let boss_hp = enemy_stats(boss).hp;
            let ratio = boss_hp / avg_hp;
            println!(
                "{} | {:?} boss HP {:.0}, avg non-boss {:.0}, ratio {:.1}x",
                era, boss, boss_hp, avg_hp, ratio
            );
            assert!(
                ratio >= 1.5,
                "{}: {:?} boss HP ratio {:.1}x < 1.5x",
                era, boss, ratio
            );
            assert!(
                ratio <= 30.0,
                "{}: {:?} boss HP ratio {:.1}x > 30x",
                era, boss, ratio
            );
        }
    }
}

#[test]
fn boss_reward_is_3_to_30x_era_average() {
    let (era_order, era_enemies) = build_era_enemies();

    for era in &era_order {
        let enemies = &era_enemies[era];
        let non_boss: Vec<EnemyType> = enemies.iter().copied().filter(|&et| !is_boss(et)).collect();
        let bosses: Vec<EnemyType> = enemies.iter().copied().filter(|&et| is_boss(et)).collect();

        if non_boss.is_empty() || bosses.is_empty() {
            continue;
        }

        let avg_gold: f32 = non_boss.iter().map(|&et| enemy_stats(et).gold_reward as f32).sum::<f32>()
            / non_boss.len() as f32;

        for &boss in &bosses {
            let boss_gold = enemy_stats(boss).gold_reward as f32;
            let ratio = boss_gold / avg_gold;
            assert!(
                ratio >= 3.0,
                "{}: {:?} boss reward ratio {:.1}x < 3x",
                era, boss, ratio
            );
            assert!(
                ratio <= 30.0,
                "{}: {:?} boss reward ratio {:.1}x > 30x",
                era, boss, ratio
            );
        }
    }
}

// ===========================================================================
// 2. Era Progression
// ===========================================================================

#[test]
fn average_non_boss_hp_generally_increases_across_eras() {
    // Era transitions can have natural dips because each era introduces new
    // lightweight scout/healer types alongside tanks. We check that the median
    // HP (more resistant to outliers than mean) trends upward with 20% tolerance.
    let (era_order, era_enemies) = build_era_enemies();
    let mut prev_median: f32 = 0.0;

    for era in &era_order {
        let enemies = &era_enemies[era];
        let non_boss: Vec<EnemyType> = enemies.iter().copied().filter(|&et| !is_boss(et)).collect();
        if non_boss.is_empty() {
            continue;
        }
        let mut hps: Vec<f32> = non_boss.iter().map(|&et| enemy_stats(et).hp).collect();
        hps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = hps[hps.len() / 2];

        println!("{}: median non-boss HP = {:.0} (of {} types)", era, median, hps.len());

        // Allow 20% tolerance for era transitions
        assert!(
            median >= prev_median * 0.8,
            "{} median HP ({:.0}) dropped >20% from previous era ({:.0})",
            era, median, prev_median
        );
        prev_median = median;
    }
}

#[test]
fn average_speed_stays_in_range() {
    let (era_order, era_enemies) = build_era_enemies();

    for era in &era_order {
        let enemies = &era_enemies[era];
        let non_boss: Vec<EnemyType> = enemies.iter().copied().filter(|&et| !is_boss(et)).collect();
        if non_boss.is_empty() {
            continue;
        }
        let avg: f32 = non_boss.iter().map(|&et| enemy_stats(et).speed).sum::<f32>()
            / non_boss.len() as f32;

        println!("{}: avg speed = {:.2}", era, avg);

        assert!(
            avg >= 0.6,
            "{} avg speed ({:.2}) < 0.6",
            era, avg
        );
        assert!(
            avg <= 4.5,
            "{} avg speed ({:.2}) > 4.5",
            era, avg
        );
    }
}

// ===========================================================================
// 3. Per-Era Archetype Coverage
// ===========================================================================

#[test]
fn each_era_has_high_armor_enemy() {
    let (era_order, era_enemies) = build_era_enemies();
    for era in &era_order {
        let has = era_enemies[era]
            .iter()
            .any(|&et| enemy_stats(et).armor >= 25.0);
        assert!(has, "{} has no enemy with armor >= 25", era);
    }
}

#[test]
fn each_era_has_magic_resistant_enemy() {
    let (era_order, era_enemies) = build_era_enemies();
    for era in &era_order {
        let has = era_enemies[era]
            .iter()
            .any(|&et| enemy_stats(et).magic_resist >= 0.1);
        assert!(has, "{} has no enemy with magic_resist >= 0.1", era);
    }
}

#[test]
fn each_era_has_flying_enemy() {
    let (era_order, era_enemies) = build_era_enemies();
    for era in &era_order {
        let has = era_enemies[era]
            .iter()
            .any(|&et| enemy_stats(et).is_flying);
        assert!(has, "{} has no flying enemy", era);
    }
}

#[test]
fn each_era_has_healer_enemy() {
    let (era_order, era_enemies) = build_era_enemies();
    for era in &era_order {
        let has = era_enemies[era]
            .iter()
            .any(|&et| enemy_stats(et).is_healer);
        assert!(has, "{} has no healer enemy", era);
    }
}

// ===========================================================================
// 4. Time-to-Kill Matrix
// ===========================================================================

#[test]
fn no_enemy_is_immune_to_any_element() {
    for &et in &ALL_ENEMY_TYPES {
        let s = enemy_stats(et);
        for &element in &ALL_ELEMENTS {
            let eff = effective_dps_for(element, 0, s.armor, s.magic_resist);
            assert!(
                eff > 0.0,
                "{:?} takes 0 effective DPS from {:?} (armor={}, mr={})",
                et, element, s.armor, s.magic_resist
            );
        }
    }
}

#[test]
fn non_boss_ttk_with_2x_l2_towers_under_60s() {
    for &et in &ALL_ENEMY_TYPES {
        if is_boss(et) {
            continue;
        }
        let s = enemy_stats(et);

        for &element in &ALL_ELEMENTS {
            // Skip hard-counter combos where the element is intentionally weak
            let ts = tower_stats(element, 2);
            let raw = ts.damage * ts.attack_speed * 2.0; // 2 towers
            let eff = if is_magic(element) {
                raw * (1.0 - s.magic_resist)
            } else {
                raw * (1.0 - s.armor / (s.armor + 100.0))
            };

            // Skip intentionally weak combos
            if !is_magic(element) && s.armor >= 40.0 { continue; }
            if element == Element::Earth && s.armor >= 25.0 { continue; }
            if is_magic(element) && s.magic_resist >= 0.25 { continue; }

            if eff <= 0.0 { continue; }

            let ttk = s.hp / eff;
            assert!(
                ttk < 60.0,
                "{:?} TTK {:.1}s vs 2x {:?} L2 (hp={}, eff_dps={:.1})",
                et, ttk, element, s.hp, eff
            );
        }
    }
}

#[test]
fn boss_ttk_with_8x_l2_towers_under_120s() {
    for &et in &ALL_ENEMY_TYPES {
        if !is_boss(et) {
            continue;
        }
        let s = enemy_stats(et);

        for &element in &ALL_ELEMENTS {
            let ts = tower_stats(element, 2);
            let raw = ts.damage * ts.attack_speed * 8.0; // 8 towers
            let eff = if is_magic(element) {
                raw * (1.0 - s.magic_resist)
            } else {
                raw * (1.0 - s.armor / (s.armor + 100.0))
            };

            // Skip physical vs 40+ armor bosses
            if !is_magic(element) && s.armor >= 40.0 { continue; }
            if eff <= 0.0 { continue; }

            let ttk = s.hp / eff;
            println!(
                "{:?} vs 8x {:?} L2 | HP: {:.0}, DPS: {:.1}, TTK: {:.1}s",
                et, element, s.hp, eff, ttk
            );
            assert!(
                ttk < 120.0,
                "{:?} TTK {:.1}s vs 8x {:?} L2 exceeds 120s",
                et, ttk, element
            );
        }
    }
}

// ===========================================================================
// 5. TTK Summary Table
// ===========================================================================

#[test]
fn ttk_summary_table() {
    println!("\n{:<20} {:<8} {:<8} {:<8} {:<8}", "Enemy", "Light", "Earth", "Ice", "Fire");
    println!("{}", "-".repeat(56));

    for &et in &ALL_ENEMY_TYPES {
        let s = enemy_stats(et);
        let ttks: Vec<String> = ALL_ELEMENTS
            .iter()
            .map(|&elem| {
                let eff = effective_dps_for(elem, 2, s.armor, s.magic_resist);
                if eff <= 0.0 {
                    "INF".to_string()
                } else {
                    format!("{:.1}s", s.hp / eff)
                }
            })
            .collect();
        println!(
            "{:<20} {:<8} {:<8} {:<8} {:<8}",
            format!("{:?}", et), ttks[0], ttks[1], ttks[2], ttks[3]
        );
    }
}
