//! Pure-logic regression tests for Ages of Aether game mechanics.
//!
//! These tests verify data integrity and game balance without using
//! Bevy ECS — they call data functions and do math directly.

use ages_of_aether::components::Element;
use ages_of_aether::data::*;

// ---------------------------------------------------------------------------
// Helper: all 4 elements
// ---------------------------------------------------------------------------

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

const FLYING_TYPES: [EnemyType; 5] = [
    EnemyType::Jellyfish,
    EnemyType::Pterodactyl,
    EnemyType::EagleScout,
    EnemyType::Wyvern,
    EnemyType::GiantEagle,
];

const HEALER_TYPES: [EnemyType; 5] = [
    EnemyType::Sporebloom,
    EnemyType::CompyHealer,
    EnemyType::Shaman,
    EnemyType::Medicus,
    EnemyType::Priest,
];

const ALL_HEROES: [HeroType; 5] = [
    HeroType::SacredMaiden,
    HeroType::IceHulk,
    HeroType::NorthernOutsider,
    HeroType::Pharaoh,
    HeroType::ScarletMagus,
];

// ===========================================================================
// 1. Enemy Stats Consistency
// ===========================================================================

#[test]
fn all_enemies_have_positive_hp_speed_gold() {
    for &et in &ALL_ENEMY_TYPES {
        let s = enemy_stats(et);
        assert!(s.hp > 0.0, "{:?} has non-positive HP: {}", et, s.hp);
        assert!(s.speed > 0.0, "{:?} has non-positive speed: {}", et, s.speed);
        assert!(s.gold_reward > 0, "{:?} has zero gold reward", et);
    }
}

#[test]
fn all_enemies_have_non_negative_armor_and_resist() {
    for &et in &ALL_ENEMY_TYPES {
        let s = enemy_stats(et);
        assert!(s.armor >= 0.0, "{:?} has negative armor: {}", et, s.armor);
        assert!(
            s.magic_resist >= 0.0 && s.magic_resist <= 1.0,
            "{:?} has magic_resist out of [0,1]: {}",
            et,
            s.magic_resist
        );
    }
}

#[test]
fn boss_enemies_have_significantly_higher_hp() {
    // Compute the average HP of non-boss enemies.
    let non_boss_hps: Vec<f32> = ALL_ENEMY_TYPES
        .iter()
        .filter(|et| !BOSS_TYPES.contains(et))
        .map(|et| enemy_stats(*et).hp)
        .collect();
    let avg_hp: f32 = non_boss_hps.iter().sum::<f32>() / non_boss_hps.len() as f32;

    for &boss in &BOSS_TYPES {
        let s = enemy_stats(boss);
        assert!(
            s.hp > avg_hp * 2.0,
            "{:?} boss HP ({}) should be > 2x average non-boss HP ({})",
            boss,
            s.hp,
            avg_hp
        );
    }
}

#[test]
fn flying_enemies_are_marked_flying() {
    for &et in &FLYING_TYPES {
        let s = enemy_stats(et);
        assert!(s.is_flying, "{:?} should be flying but is_flying=false", et);
    }
}

#[test]
fn non_flying_enemies_are_not_flying() {
    let ground_types: Vec<EnemyType> = ALL_ENEMY_TYPES
        .iter()
        .copied()
        .filter(|et| !FLYING_TYPES.contains(et))
        .collect();
    for et in ground_types {
        let s = enemy_stats(et);
        assert!(!s.is_flying, "{:?} should NOT be flying but is_flying=true", et);
    }
}

#[test]
fn healer_enemies_are_marked_healer() {
    for &et in &HEALER_TYPES {
        let s = enemy_stats(et);
        assert!(s.is_healer, "{:?} should be a healer but is_healer=false", et);
    }
}

#[test]
fn non_healer_enemies_are_not_healer() {
    let non_healers: Vec<EnemyType> = ALL_ENEMY_TYPES
        .iter()
        .copied()
        .filter(|et| !HEALER_TYPES.contains(et))
        .collect();
    for et in non_healers {
        let s = enemy_stats(et);
        assert!(!s.is_healer, "{:?} should NOT be a healer but is_healer=true", et);
    }
}

#[test]
fn all_enemies_have_valid_model_path() {
    for &et in &ALL_ENEMY_TYPES {
        let s = enemy_stats(et);
        assert!(
            !s.model_path.is_empty(),
            "{:?} has empty model_path",
            et
        );
        assert!(
            s.model_scale > 0.0,
            "{:?} has non-positive model_scale: {}",
            et,
            s.model_scale
        );
    }
}

// ===========================================================================
// 2. Tower Stats Consistency
// ===========================================================================

#[test]
fn all_tower_element_level_combos_have_valid_stats() {
    for &element in &ALL_ELEMENTS {
        for level in 0..3u8 {
            let s = tower_stats(element, level);
            assert!(s.cost > 0, "{:?} lv{} has zero cost", element, level);
            assert!(s.damage > 0.0, "{:?} lv{} has non-positive damage", element, level);
            assert!(s.attack_speed > 0.0, "{:?} lv{} has non-positive attack_speed", element, level);
            assert!(s.range > 0.0, "{:?} lv{} has non-positive range", element, level);
            assert!(!s.model_path.is_empty(), "{:?} lv{} has empty model_path", element, level);
            assert!(!s.name.is_empty(), "{:?} lv{} has empty name", element, level);
        }
    }
}

#[test]
fn tower_damage_increases_with_level() {
    for &element in &ALL_ELEMENTS {
        let d0 = tower_stats(element, 0).damage;
        let d1 = tower_stats(element, 1).damage;
        let d2 = tower_stats(element, 2).damage;
        assert!(
            d1 > d0,
            "{:?} lv1 damage ({}) should exceed lv0 ({})",
            element, d1, d0
        );
        assert!(
            d2 > d1,
            "{:?} lv2 damage ({}) should exceed lv1 ({})",
            element, d2, d1
        );
    }
}

#[test]
fn tower_cost_increases_with_level() {
    for &element in &ALL_ELEMENTS {
        let c0 = tower_stats(element, 0).cost;
        let c1 = tower_stats(element, 1).cost;
        let c2 = tower_stats(element, 2).cost;
        assert!(
            c1 > c0,
            "{:?} lv1 cost ({}) should exceed lv0 ({})",
            element, c1, c0
        );
        assert!(
            c2 > c1,
            "{:?} lv2 cost ({}) should exceed lv1 ({})",
            element, c2, c1
        );
    }
}

#[test]
fn tower_attack_speed_non_decreasing_with_level() {
    for &element in &ALL_ELEMENTS {
        let a0 = tower_stats(element, 0).attack_speed;
        let a1 = tower_stats(element, 1).attack_speed;
        let a2 = tower_stats(element, 2).attack_speed;
        assert!(
            a1 >= a0,
            "{:?} lv1 attack_speed ({}) should be >= lv0 ({})",
            element, a1, a0
        );
        assert!(
            a2 >= a1,
            "{:?} lv2 attack_speed ({}) should be >= lv1 ({})",
            element, a2, a1
        );
    }
}

#[test]
fn tower_range_non_decreasing_with_level() {
    for &element in &ALL_ELEMENTS {
        let r0 = tower_stats(element, 0).range;
        let r1 = tower_stats(element, 1).range;
        let r2 = tower_stats(element, 2).range;
        assert!(
            r1 >= r0,
            "{:?} lv1 range ({}) should be >= lv0 ({})",
            element, r1, r0
        );
        assert!(
            r2 >= r1,
            "{:?} lv2 range ({}) should be >= lv1 ({})",
            element, r2, r1
        );
    }
}

#[test]
fn tower_base_cost_matches_level_zero() {
    for &element in &ALL_ELEMENTS {
        assert_eq!(
            tower_base_cost(element),
            tower_stats(element, 0).cost,
            "{:?} tower_base_cost mismatch with tower_stats lv0",
            element
        );
    }
}

#[test]
fn each_element_has_two_specializations() {
    for &element in &ALL_ELEMENTS {
        let specs = element_specializations(element);
        assert_eq!(specs.len(), 2, "{:?} should have exactly 2 specializations", element);
        for (_, def) in &specs {
            assert!(def.cost > 0, "Specialization '{}' has zero cost", def.name);
            assert!(!def.name.is_empty(), "Specialization has empty name");
            assert!(!def.description.is_empty(), "Specialization '{}' has empty description", def.name);
        }
    }
}

// ===========================================================================
// 3. Damage Calculation Tests
// ===========================================================================

/// Physical damage formula: effective = damage * (1.0 - armor / (armor + 100.0))
fn physical_damage(raw: f32, armor: f32) -> f32 {
    raw * (1.0 - armor / (armor + 100.0))
}

/// Magic damage formula: effective = damage * (1.0 - magic_resist)
fn magic_damage(raw: f32, magic_resist: f32) -> f32 {
    raw * (1.0 - magic_resist)
}

#[test]
fn zero_armor_means_full_physical_damage() {
    let dmg = physical_damage(100.0, 0.0);
    assert!(
        (dmg - 100.0).abs() < f32::EPSILON,
        "0 armor should give full damage, got {}",
        dmg
    );
}

#[test]
fn hundred_armor_halves_physical_damage() {
    let dmg = physical_damage(100.0, 100.0);
    assert!(
        (dmg - 50.0).abs() < f32::EPSILON,
        "100 armor should give 50% damage, got {}",
        dmg
    );
}

#[test]
fn physical_damage_diminishing_returns() {
    // Going from 100 to 200 armor should give less reduction than 0 to 100.
    let reduction_0_to_100 = 100.0 - physical_damage(100.0, 100.0);
    let reduction_100_to_200 = physical_damage(100.0, 100.0) - physical_damage(100.0, 200.0);
    assert!(
        reduction_0_to_100 > reduction_100_to_200,
        "Armor should have diminishing returns: first 100 reduces by {}, next 100 by {}",
        reduction_0_to_100,
        reduction_100_to_200
    );
}

#[test]
fn zero_magic_resist_means_full_magic_damage() {
    let dmg = magic_damage(100.0, 0.0);
    assert!(
        (dmg - 100.0).abs() < f32::EPSILON,
        "0 magic_resist should give full damage, got {}",
        dmg
    );
}

#[test]
fn thirty_percent_magic_resist_gives_seventy_percent_damage() {
    let dmg = magic_damage(100.0, 0.3);
    assert!(
        (dmg - 70.0).abs() < 0.01,
        "0.3 magic_resist should give ~70 damage, got {}",
        dmg
    );
}

#[test]
fn magic_damage_scales_linearly_with_resist() {
    let dmg_20 = magic_damage(100.0, 0.2);
    let dmg_40 = magic_damage(100.0, 0.4);
    // Reduction from 0.2 to 0.4 should be the same as 0.0 to 0.2.
    let r1 = 100.0 - dmg_20;
    let r2 = dmg_20 - dmg_40;
    assert!(
        (r1 - r2).abs() < 0.01,
        "Magic resist should scale linearly: delta1={}, delta2={}",
        r1,
        r2
    );
}

#[test]
fn physical_damage_against_real_enemies() {
    // Verify the formula applied to actual enemy armor values.
    let triceratops = enemy_stats(EnemyType::Triceratops);
    let raw = 100.0;
    let eff = physical_damage(raw, triceratops.armor);
    assert!(
        eff < raw,
        "Triceratops (armor={}) should reduce damage below raw",
        triceratops.armor
    );
    assert!(
        eff > 0.0,
        "Even armored enemies should take some physical damage"
    );
}

#[test]
fn magic_damage_against_real_enemies() {
    // Verify for an enemy with nonzero magic resist.
    let nautilus = enemy_stats(EnemyType::Nautilus);
    let raw = 100.0;
    let eff = magic_damage(raw, nautilus.magic_resist);
    let expected = raw * (1.0 - nautilus.magic_resist);
    assert!(
        (eff - expected).abs() < 0.01,
        "Magic damage vs Nautilus: expected {}, got {}",
        expected,
        eff
    );
}

// ===========================================================================
// 4. Level Data Integrity
// ===========================================================================

#[test]
fn all_levels_have_valid_paths() {
    for level in 1..=MAX_LEVELS {
        let path = level_path(level);
        assert!(
            path.len() >= 2,
            "Level {} path has fewer than 2 waypoints (got {})",
            level,
            path.len()
        );
    }
}

#[test]
fn all_levels_have_enough_build_spots() {
    for level in 1..=MAX_LEVELS {
        let spots = level_build_spots(level);
        assert!(
            spots.len() >= 6,
            "Level {} has fewer than 6 build spots (got {})",
            level,
            spots.len()
        );
    }
}

#[test]
fn all_levels_have_ten_waves() {
    for level in 1..=MAX_LEVELS {
        let waves = level_waves(level);
        assert_eq!(
            waves.len(),
            10,
            "Level {} should have 10 waves, got {}",
            level,
            waves.len()
        );
    }
}

#[test]
fn all_wave_groups_reference_valid_enemy_types() {
    // If an enemy type were invalid, enemy_stats would panic (unreachable).
    // We also verify that count/interval are sensible.
    for level in 1..=MAX_LEVELS {
        let waves = level_waves(level);
        for (wi, wave) in waves.iter().enumerate() {
            assert!(
                !wave.groups.is_empty(),
                "Level {} wave {} has no groups",
                level,
                wi + 1
            );
            for group in &wave.groups {
                // This call would panic if enemy_type were somehow not handled:
                let _ = enemy_stats(group.enemy_type);
                assert!(
                    group.count > 0,
                    "Level {} wave {} has a group with count=0",
                    level,
                    wi + 1
                );
                assert!(
                    group.interval > 0.0,
                    "Level {} wave {} has a group with non-positive interval",
                    level,
                    wi + 1
                );
            }
        }
    }
}

#[test]
fn starting_gold_can_afford_at_least_two_towers() {
    // The cheapest tower is the minimum across all elements at level 0.
    let cheapest = ALL_ELEMENTS
        .iter()
        .map(|&e| tower_stats(e, 0).cost)
        .min()
        .unwrap();

    for level in 1..=MAX_LEVELS {
        let cfg = level_start_config(level);
        assert!(
            cfg.starting_gold >= cheapest * 2,
            "Level {} starting_gold ({}) cannot afford 2 of the cheapest tower (cost {})",
            level,
            cfg.starting_gold,
            cheapest
        );
    }
}

#[test]
fn level_start_configs_are_valid() {
    for level in 1..=MAX_LEVELS {
        let cfg = level_start_config(level);
        assert!(cfg.starting_gold > 0, "Level {} starting_gold is 0", level);
        assert!(cfg.lives > 0, "Level {} lives is 0", level);
        assert_eq!(cfg.max_waves, 10, "Level {} max_waves should be 10", level);
        assert!(cfg.wave_hp_scale > 0.0, "Level {} wave_hp_scale <= 0", level);
        assert!(cfg.wave_speed_scale > 0.0, "Level {} wave_speed_scale <= 0", level);
    }
}

#[test]
fn level_info_names_are_non_empty() {
    for level in 1..=MAX_LEVELS {
        let info = level_info(level);
        assert!(!info.name.is_empty(), "Level {} has empty name", level);
        assert!(!info.era.is_empty(), "Level {} has empty era", level);
        assert!(!info.description.is_empty(), "Level {} has empty description", level);
        assert_eq!(info.waves, 10, "Level {} info.waves should be 10", level);
    }
}

#[test]
fn path_waypoints_are_not_degenerate() {
    // Each consecutive pair of waypoints should have some distance between them.
    for level in 1..=MAX_LEVELS {
        let path = level_path(level);
        for i in 1..path.len() {
            let d = (path[i] - path[i - 1]).length();
            assert!(
                d > 0.1,
                "Level {} waypoints {} and {} are too close (dist={})",
                level,
                i - 1,
                i,
                d
            );
        }
    }
}

// ===========================================================================
// 5. Economy Balance Sanity
// ===========================================================================

#[test]
fn sell_refund_rate_is_sixty_percent() {
    assert!(
        (SELL_REFUND_RATE - 0.6).abs() < f32::EPSILON,
        "SELL_REFUND_RATE should be 0.6, got {}",
        SELL_REFUND_RATE
    );
}

#[test]
fn sell_refund_of_full_upgrade_path() {
    // Total investment = sum of cost at each level. Refund = 60%.
    for &element in &ALL_ELEMENTS {
        let total_investment: u32 = (0..3u8).map(|lv| tower_stats(element, lv).cost).sum();
        let refund = (total_investment as f32 * SELL_REFUND_RATE) as u32;
        assert!(
            refund > 0,
            "{:?} fully upgraded sell refund should be positive",
            element
        );
        assert!(
            refund < total_investment,
            "{:?} sell refund ({}) should be less than investment ({})",
            element,
            refund,
            total_investment
        );
    }
}

#[test]
fn upgrade_costs_are_strictly_positive() {
    for &element in &ALL_ELEMENTS {
        for level in 0..3u8 {
            let cost = tower_stats(element, level).cost;
            assert!(cost > 0, "{:?} lv{} cost should be > 0", element, level);
        }
    }
}

#[test]
fn wave_enemy_gold_reward_is_reasonable() {
    // For each wave across all levels, the total gold from killing all enemies
    // should be between 10 and 5000 (sanity bounds).
    for level in 1..=MAX_LEVELS {
        let waves = level_waves(level);
        for (wi, wave) in waves.iter().enumerate() {
            let total_gold: u32 = wave
                .groups
                .iter()
                .map(|g| g.count * enemy_stats(g.enemy_type).gold_reward)
                .sum();
            assert!(
                total_gold >= 10,
                "Level {} wave {} total gold ({}) seems too low",
                level,
                wi + 1,
                total_gold
            );
            assert!(
                total_gold <= 5000,
                "Level {} wave {} total gold ({}) seems unreasonably high",
                level,
                wi + 1,
                total_gold
            );
        }
    }
}

#[test]
fn early_call_bonus_is_reasonable() {
    for level in 1..=MAX_LEVELS {
        let waves = level_waves(level);
        for (wi, wave) in waves.iter().enumerate() {
            // Early call bonus should exist but not be absurdly high.
            assert!(
                wave.early_call_bonus <= 500,
                "Level {} wave {} early_call_bonus ({}) is unreasonably large",
                level,
                wi + 1,
                wave.early_call_bonus
            );
        }
    }
}

// ===========================================================================
// 6. Hero Stats
// ===========================================================================

#[test]
fn all_heroes_have_positive_stats() {
    for &hero in &ALL_HEROES {
        let s = hero_stats(hero);
        assert!(s.hp > 0.0, "{:?} has non-positive HP: {}", hero, s.hp);
        assert!(s.damage > 0.0, "{:?} has non-positive damage: {}", hero, s.damage);
        assert!(
            s.attack_speed > 0.0,
            "{:?} has non-positive attack_speed: {}",
            hero,
            s.attack_speed
        );
        assert!(
            s.attack_range > 0.0,
            "{:?} has non-positive attack_range: {}",
            hero,
            s.attack_range
        );
        assert!(
            s.move_speed > 0.0,
            "{:?} has non-positive move_speed: {}",
            hero,
            s.move_speed
        );
        assert!(
            s.respawn_time > 0.0,
            "{:?} has non-positive respawn_time: {}",
            hero,
            s.respawn_time
        );
    }
}

#[test]
fn all_heroes_have_valid_model() {
    for &hero in &ALL_HEROES {
        let s = hero_stats(hero);
        assert!(!s.model_path.is_empty(), "{:?} has empty model_path", hero);
        assert!(
            s.model_scale > 0.0,
            "{:?} has non-positive model_scale: {}",
            hero,
            s.model_scale
        );
    }
}

#[test]
fn all_heroes_have_animation_paths() {
    for &hero in &ALL_HEROES {
        let s = hero_stats(hero);
        assert!(!s.idle_anim.is_empty(), "{:?} has empty idle_anim", hero);
        assert!(!s.attack_anim.is_empty(), "{:?} has empty attack_anim", hero);
        assert!(!s.run_anim.is_empty(), "{:?} has empty run_anim", hero);
    }
}

#[test]
fn all_heroes_have_three_abilities() {
    for &hero in &ALL_HEROES {
        let abilities = hero_abilities(hero);
        assert_eq!(
            abilities.len(),
            3,
            "{:?} should have exactly 3 abilities",
            hero
        );
        for (i, ability) in abilities.iter().enumerate() {
            assert!(
                !ability.name.is_empty(),
                "{:?} ability {} has empty name",
                hero,
                i
            );
            assert!(
                ability.cooldown > 0.0,
                "{:?} ability '{}' has non-positive cooldown: {}",
                hero,
                ability.name,
                ability.cooldown
            );
        }
    }
}

#[test]
fn hero_names_are_non_empty() {
    for &hero in &ALL_HEROES {
        let s = hero_stats(hero);
        assert!(!s.name.is_empty(), "{:?} has empty name", hero);
    }
}

#[test]
fn hero_names_are_unique() {
    let names: Vec<&str> = ALL_HEROES.iter().map(|&h| hero_stats(h).name).collect();
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            assert_ne!(
                names[i], names[j],
                "Heroes {:?} and {:?} share the same name '{}'",
                ALL_HEROES[i], ALL_HEROES[j], names[i]
            );
        }
    }
}
