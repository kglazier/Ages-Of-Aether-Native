//! Tower DPS benchmarks and cross-element balance tests.
//!
//! Ported from the TypeScript towers.test.ts with improvements:
//! - Upgrade efficiency checks (DPS-per-gold must not collapse)
//! - Cross-element balance band (no element > 4x another at L0)
//! - Cost-to-DPS-gain ratio cap

use ages_of_aether::components::Element;
use ages_of_aether::data::*;

const ALL_ELEMENTS: [Element; 4] = [
    Element::Lightning,
    Element::Earth,
    Element::Ice,
    Element::Fire,
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn raw_dps(element: Element, level: u8) -> f32 {
    let s = tower_stats(element, level);
    s.damage * s.attack_speed
}

fn physical_effective_dps(raw: f32, armor: f32) -> f32 {
    raw * (1.0 - armor / (armor + 100.0))
}

fn magic_effective_dps(raw: f32, mr: f32) -> f32 {
    raw * (1.0 - mr)
}

// ===========================================================================
// 1. DPS Benchmarks — every element/level combo produces positive DPS
// ===========================================================================

#[test]
fn all_tower_dps_positive() {
    for &element in &ALL_ELEMENTS {
        for level in 0..3u8 {
            let dps = raw_dps(element, level);
            let s = tower_stats(element, level);
            println!(
                "{:?} L{} ({}) | DPS: {:.1} (dmg: {:.1} x spd: {:.2})",
                element, level, s.name, dps, s.damage, s.attack_speed
            );
            assert!(dps > 0.0, "{:?} L{} has zero/negative DPS", element, level);
        }
    }
}

// ===========================================================================
// 2. Effective DPS against armor/MR
// ===========================================================================

#[test]
fn physical_towers_effective_dps_against_armor() {
    for &element in &[Element::Earth, Element::Fire] {
        for level in 0..3u8 {
            let dps = raw_dps(element, level);
            for armor in [0.0, 25.0, 50.0] {
                let eff = physical_effective_dps(dps, armor);
                assert!(
                    eff > 0.0,
                    "{:?} L{} vs {} armor should deal positive DPS (got {:.1})",
                    element, level, armor, eff
                );
            }
        }
    }
}

#[test]
fn magic_towers_effective_dps_against_mr() {
    for &element in &[Element::Lightning, Element::Ice] {
        for level in 0..3u8 {
            let dps = raw_dps(element, level);
            for mr in [0.0, 0.2, 0.4] {
                let eff = magic_effective_dps(dps, mr);
                assert!(
                    eff > 0.0,
                    "{:?} L{} vs {:.0}% MR should deal positive DPS (got {:.1})",
                    element, level, mr * 100.0, eff
                );
            }
        }
    }
}

// ===========================================================================
// 3. Upgrade Efficiency — each level's DPS-per-gold should not collapse
// ===========================================================================

#[test]
fn upgrade_efficiency_does_not_collapse() {
    for &element in &ALL_ELEMENTS {
        for level in 1..3u8 {
            let prev = tower_stats(element, level - 1);
            let curr = tower_stats(element, level);
            let prev_dps = prev.damage * prev.attack_speed;
            let curr_dps = curr.damage * curr.attack_speed;
            let prev_eff = prev_dps / prev.cost as f32;
            let curr_eff = curr_dps / curr.cost as f32;

            // Each upgrade's DPS-per-gold should be at least 50% of the previous level
            assert!(
                curr_eff >= prev_eff * 0.5,
                "{:?} L{} DPS/gold ({:.4}) < 50% of L{} ({:.4})",
                element, level, curr_eff, level - 1, prev_eff
            );

            // DPS should actually increase
            let dps_gain = curr_dps - prev_dps;
            assert!(
                dps_gain > 0.0,
                "{:?} L{} DPS ({:.1}) does not exceed L{} ({:.1})",
                element, level, curr_dps, level - 1, prev_dps
            );

            // Cost-to-DPS-gain ratio: upgrade cost / DPS gain should be <= 3x the base ratio
            let base_ratio = prev.cost as f32 / prev_dps;
            let upgrade_ratio = curr.cost as f32 / dps_gain;
            assert!(
                upgrade_ratio <= base_ratio * 3.0,
                "{:?} L{} upgrade ratio ({:.1}) > 3x base ratio ({:.1})",
                element, level, upgrade_ratio, base_ratio
            );
        }
    }
}

// ===========================================================================
// 4. Cross-Element Balance
// ===========================================================================

#[test]
fn no_element_exceeds_4x_dps_per_gold_of_another_at_l0() {
    // Earth towers have intentionally low raw DPS (golem blocking value).
    // Fire has low attack speed but AoE. Allow wider band for utility trade-offs.
    let efficiencies: Vec<(Element, f32)> = ALL_ELEMENTS
        .iter()
        .map(|&e| {
            let s = tower_stats(e, 0);
            let dps = s.damage * s.attack_speed;
            (e, dps / s.cost as f32)
        })
        .collect();

    let max = efficiencies.iter().map(|(_, e)| *e).fold(0.0f32, f32::max);
    let min = efficiencies.iter().map(|(_, e)| *e).fold(f32::MAX, f32::min);

    println!("L0 DPS-per-gold:");
    for (elem, eff) in &efficiencies {
        println!("  {:?}: {:.4}", elem, eff);
    }
    println!("  Spread: {:.2}x", max / min);

    assert!(
        max / min <= 4.0,
        "L0 DPS-per-gold spread ({:.2}x) exceeds 4x",
        max / min
    );
}

#[test]
fn total_cost_to_max_each_element_within_2x() {
    let total_costs: Vec<(Element, u32)> = ALL_ELEMENTS
        .iter()
        .map(|&e| {
            let total: u32 = (0..3u8).map(|lv| tower_stats(e, lv).cost).sum();
            (e, total)
        })
        .collect();

    let max = total_costs.iter().map(|(_, c)| *c).max().unwrap();
    let min = total_costs.iter().map(|(_, c)| *c).min().unwrap();

    println!("Total upgrade cost per element:");
    for (elem, cost) in &total_costs {
        println!("  {:?}: {}", elem, cost);
    }
    println!("  Spread: {:.2}x", max as f32 / min as f32);

    assert!(
        max as f32 / min as f32 <= 2.0,
        "Total cost spread ({:.2}x) exceeds 2x",
        max as f32 / min as f32
    );
}

// ===========================================================================
// 5. DPS Summary Table
// ===========================================================================

#[test]
fn dps_summary_table() {
    println!("\n{:<12} {:<6} {:<12} {:<8} {:<8} {:<10}", "Element", "Level", "Name", "DPS", "Cost", "DPS/Gold");
    println!("{}", "-".repeat(60));
    for &element in &ALL_ELEMENTS {
        for level in 0..3u8 {
            let s = tower_stats(element, level);
            let dps = s.damage * s.attack_speed;
            let eff = dps / s.cost as f32;
            println!(
                "{:<12} {:<6} {:<12} {:<8.1} {:<8} {:<10.4}",
                format!("{:?}", element), level, s.name, dps, s.cost, eff
            );
        }
    }
}
