//! Headless balance simulation tests for Ages of Aether.
//!
//! These tests simulate game play-throughs at three AI difficulty levels
//! (easy / medium / hard) across all 10 levels and report whether each
//! level is beatable, lives remaining, star ratings, and gold efficiency.
//!
//! No Bevy App or rendering is involved — all calculations are pure math
//! driven by the data module.

use ages_of_aether::components::Element;
use ages_of_aether::data::*;

// ---------------------------------------------------------------------------
// Constants & helpers
// ---------------------------------------------------------------------------

const SELL_REFUND: f32 = 0.6;

/// Star rating based on fraction of starting lives remaining.
fn star_rating(lives: u32, max_lives: u32) -> u32 {
    if lives == 0 {
        return 0;
    }
    let frac = lives as f32 / max_lives as f32;
    if frac >= 0.9 {
        3
    } else if frac >= 0.5 {
        2
    } else {
        1
    }
}

fn star_str(stars: u32) -> &'static str {
    match stars {
        3 => "***",
        2 => "** ",
        1 => "*  ",
        _ => "X  ",
    }
}

/// Compute the total length of a path (sum of segment distances).
fn path_length(path: &[bevy::math::Vec3]) -> f32 {
    path.windows(2)
        .map(|w| (w[1] - w[0]).length())
        .sum()
}

// ---------------------------------------------------------------------------
// Effective DPS helpers
// ---------------------------------------------------------------------------

/// Raw DPS for a tower (damage * attack_speed).
fn raw_dps(element: Element, level: u8) -> f32 {
    let s = tower_stats(element, level);
    s.damage * s.attack_speed
}

/// Physical damage multiplier (Earth, Fire) against armor.
fn phys_mult(armor: f32) -> f32 {
    1.0 - armor / (armor + 100.0)
}

/// Magic damage multiplier (Lightning, Ice) against magic resist.
fn magic_mult(mr: f32) -> f32 {
    1.0 - mr
}

/// Whether an element deals magic damage.
fn is_magic(element: Element) -> bool {
    matches!(element, Element::Lightning | Element::Ice)
}

/// Effective DPS of one tower against an enemy, including synergy bonuses.
/// `has_slow` is true when at least one Ice tower is present in the defense.
fn effective_dps(element: Element, level: u8, armor: f32, mr: f32, has_slow: bool) -> f32 {
    let base = raw_dps(element, level);
    let after_resist = if is_magic(element) {
        base * magic_mult(mr)
    } else {
        base * phys_mult(armor)
    };

    // Synergy bonuses (true damage, ignores resist) — only apply when enemies are slowed.
    let synergy = if has_slow {
        match element {
            // Ice + Lightning synergy: +50% of base as true damage
            Element::Lightning => base * 0.5,
            // Ice + Fire synergy: +40% of base as true damage
            Element::Fire => base * 0.4,
            _ => 0.0,
        }
    } else {
        0.0
    };

    // Fire bonus: AoE splash (50%) + burn (3 dps for 3 sec = 9 total, amortised to ~3 dps)
    let fire_extra = if element == Element::Fire {
        // splash hits ~1 extra enemy on average  => +50% effective
        // burn adds ~3 dps true damage
        after_resist * 0.5 + 3.0
    } else {
        0.0
    };

    // Ice 50% slow effectively doubles the time enemies are in range for all towers,
    // but we model that in the kill-time calculation instead.

    after_resist + synergy + fire_extra
}

// ---------------------------------------------------------------------------
// Tower placement state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct PlacedTower {
    element: Element,
    level: u8,
    total_invested: u32,
}

impl PlacedTower {
    fn new(element: Element) -> Self {
        Self {
            element,
            level: 0,
            total_invested: tower_stats(element, 0).cost,
        }
    }

    fn upgrade_cost(&self) -> Option<u32> {
        if self.level >= 2 {
            return None;
        }
        Some(tower_stats(self.element, self.level + 1).cost)
    }

    fn sell_value(&self) -> u32 {
        (self.total_invested as f32 * SELL_REFUND) as u32
    }
}

// ---------------------------------------------------------------------------
// AI strategies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Strategy {
    Easy,
    Medium,
    Hard,
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strategy::Easy => write!(f, "Easy"),
            Strategy::Medium => write!(f, "Medium"),
            Strategy::Hard => write!(f, "Hard"),
        }
    }
}

/// Between-wave spending phase. Returns updated gold.
fn ai_spend(
    strategy: Strategy,
    gold: &mut u32,
    towers: &mut Vec<PlacedTower>,
    max_spots: usize,
) {
    match strategy {
        // -----------------------------------------------------------------
        // EASY: only Lightning, never upgrades, fills left to right
        // -----------------------------------------------------------------
        Strategy::Easy => {
            let cost = tower_stats(Element::Lightning, 0).cost;
            while towers.len() < max_spots && *gold >= cost {
                *gold -= cost;
                towers.push(PlacedTower::new(Element::Lightning));
            }
        }

        // -----------------------------------------------------------------
        // MEDIUM: Lightning + Ice mix, upgrades to L1 when affordable
        // -----------------------------------------------------------------
        Strategy::Medium => {
            // First: try to upgrade existing towers to L1
            for t in towers.iter_mut() {
                if t.level == 0 {
                    if let Some(cost) = t.upgrade_cost() {
                        if *gold >= cost {
                            *gold -= cost;
                            t.total_invested += cost;
                            t.level = 1;
                        }
                    }
                }
            }
            // Then: build new towers (alternate Lightning, Ice)
            while towers.len() < max_spots {
                // Decide element: keep roughly 2:1 Lightning:Ice ratio
                let ice_count = towers.iter().filter(|t| t.element == Element::Ice).count();
                let light_count = towers.iter().filter(|t| t.element == Element::Lightning).count();
                let element = if ice_count * 2 < light_count {
                    Element::Ice
                } else {
                    Element::Lightning
                };
                let cost = tower_stats(element, 0).cost;
                if *gold >= cost {
                    *gold -= cost;
                    towers.push(PlacedTower::new(element));
                } else {
                    break;
                }
            }
        }

        // -----------------------------------------------------------------
        // HARD: all 4 elements, upgrades to L2, sells underperformers
        // -----------------------------------------------------------------
        Strategy::Hard => {
            // Phase 1: Upgrade best towers to L2
            // Prioritise Ice first (enables synergies), then Lightning, then Fire
            let upgrade_order = [Element::Ice, Element::Lightning, Element::Fire, Element::Earth];
            for &elem in &upgrade_order {
                for t in towers.iter_mut() {
                    if t.element == elem && t.level < 2 {
                        if let Some(cost) = t.upgrade_cost() {
                            if *gold >= cost {
                                *gold -= cost;
                                t.total_invested += cost;
                                t.level += 1;
                            }
                        }
                    }
                }
            }

            // Phase 2: Build new towers with strategic mix
            // Target composition: 2 Ice, 3 Lightning, 2 Fire, rest Earth
            while towers.len() < max_spots {
                let ice_n = towers.iter().filter(|t| t.element == Element::Ice).count();
                let light_n = towers.iter().filter(|t| t.element == Element::Lightning).count();
                let fire_n = towers.iter().filter(|t| t.element == Element::Fire).count();

                let element = if ice_n < 2 {
                    Element::Ice
                } else if light_n < 3 {
                    Element::Lightning
                } else if fire_n < 2 {
                    Element::Fire
                } else {
                    Element::Earth
                };

                let cost = tower_stats(element, 0).cost;
                if *gold >= cost {
                    *gold -= cost;
                    towers.push(PlacedTower::new(element));
                } else {
                    break;
                }
            }

            // Phase 3: If all spots full, sell lowest-level Earth towers to fund upgrades
            if towers.len() >= max_spots {
                // Find cheapest un-upgraded tower
                let sell_idx = towers
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.level == 0 && t.element == Element::Earth)
                    .map(|(i, t)| (i, t.sell_value()))
                    .max_by_key(|(_, v)| *v)
                    .map(|(i, _)| i);
                if let Some(idx) = sell_idx {
                    let refund = towers[idx].sell_value();
                    *gold += refund;
                    towers.remove(idx);
                    // Try to upgrade an important tower with the freed gold
                    for &elem in &[Element::Ice, Element::Lightning, Element::Fire] {
                        for t in towers.iter_mut() {
                            if t.element == elem && t.level < 2 {
                                if let Some(cost) = t.upgrade_cost() {
                                    if *gold >= cost {
                                        *gold -= cost;
                                        t.total_invested += cost;
                                        t.level += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Wave simulation
// ---------------------------------------------------------------------------

struct SimResult {
    victory: bool,
    lives_remaining: u32,
    max_lives: u32,
    stars: u32,
    gold_earned: u32,
    gold_spent: u32,
    waves_survived: u32,
}

fn simulate_level(level: u32, strategy: Strategy) -> SimResult {
    let config = level_start_config(level);
    let path = level_path(level);
    let spots = level_build_spots(level);
    let waves = level_waves(level);
    let total_path_len = path_length(&path);
    let max_spots = spots.len();

    let mut gold = config.starting_gold;
    let mut lives = config.lives;
    let max_lives = config.lives;
    let mut towers: Vec<PlacedTower> = Vec::new();
    let mut total_gold_earned: u32 = 0;
    let mut total_gold_spent: u32 = 0;
    let mut waves_survived: u32 = 0;

    // Initial build phase
    let gold_before = gold;
    ai_spend(strategy, &mut gold, &mut towers, max_spots);
    total_gold_spent += gold_before - gold;

    for (wave_idx, wave_def) in waves.iter().enumerate() {
        let wave_num = (wave_idx + 1) as u32;

        // --- Check if any Ice tower is present for slow synergy ---
        let has_slow = towers.iter().any(|t| t.element == Element::Ice);

        // --- Process each group in the wave ---
        let mut wave_gold_reward: u32 = 0;
        let mut wave_leaks: u32 = 0;

        for group in &wave_def.groups {
            let stats = enemy_stats(group.enemy_type);

            // Scale HP and speed by wave number
            let scaled_hp =
                stats.hp * (1.0 + config.wave_hp_scale * (wave_num as f32 - 1.0));
            let scaled_speed =
                stats.speed * (1.0 + config.wave_speed_scale * (wave_num as f32 - 1.0));

            // Time for one enemy to traverse the full path
            let traverse_time = total_path_len / scaled_speed;

            // How much damage can all towers deal to one enemy while it's in range?
            // Average tower range is ~5.5 units; time in range per tower = range / speed.
            // But multiple towers fire simultaneously, so total damage = sum(dps_i * time_in_range_i).
            // For simplicity: each tower's effective range time = min(tower_range / speed, traverse_time).
            let mut total_damage_per_enemy: f32 = 0.0;
            for t in &towers {
                let t_stats = tower_stats(t.element, t.level);
                let time_in_range = (t_stats.range / scaled_speed).min(traverse_time);
                // Slow debuff from Ice: enemies move 50% slower => 2x time in range
                let slow_factor = if has_slow && t.element != Element::Ice {
                    // Ice tower itself applies the slow, so its own time_in_range
                    // benefits too, but conservatively use 1.5x for Ice tower
                    1.5
                } else if has_slow {
                    1.5
                } else {
                    1.0
                };
                let dps = effective_dps(t.element, t.level, stats.armor, stats.magic_resist, has_slow);
                total_damage_per_enemy += dps * time_in_range * slow_factor;
            }

            // How many enemies survive the gauntlet?
            let kill_fraction = (total_damage_per_enemy / scaled_hp).min(1.0);
            let enemies_killed =
                (group.count as f32 * kill_fraction).floor() as u32;
            let enemies_leaked = group.count.saturating_sub(enemies_killed);

            // Flying enemies: Earth towers can't target them (no anti-air for barracks).
            // Adjust: remove Earth tower contributions for flying enemies.
            let (enemies_killed, enemies_leaked) = if stats.is_flying {
                let mut flying_dmg: f32 = 0.0;
                for t in &towers {
                    if t.element == Element::Earth {
                        continue; // Earth can't hit flyers
                    }
                    let t_stats = tower_stats(t.element, t.level);
                    let time_in_range = (t_stats.range / scaled_speed).min(traverse_time);
                    let slow_factor = if has_slow { 1.5 } else { 1.0 };
                    let dps = effective_dps(
                        t.element, t.level, stats.armor, stats.magic_resist, has_slow,
                    );
                    flying_dmg += dps * time_in_range * slow_factor;
                }
                let frac = (flying_dmg / scaled_hp).min(1.0);
                let killed = (group.count as f32 * frac).floor() as u32;
                let leaked = group.count.saturating_sub(killed);
                (killed, leaked)
            } else {
                (enemies_killed, enemies_leaked)
            };

            wave_gold_reward += enemies_killed * stats.gold_reward;
            wave_leaks += enemies_leaked;
        }

        // Subtract leaked enemies from lives
        if wave_leaks > 0 {
            lives = lives.saturating_sub(wave_leaks);
        }

        // Award gold (kills + early call bonus for simplicity)
        gold += wave_gold_reward + wave_def.early_call_bonus;
        total_gold_earned += wave_gold_reward + wave_def.early_call_bonus;

        waves_survived = wave_num;

        if lives == 0 {
            break;
        }

        // Between-wave spending
        let gold_before = gold;
        ai_spend(strategy, &mut gold, &mut towers, max_spots);
        total_gold_spent += gold_before - gold;
    }

    let stars = star_rating(lives, max_lives);
    SimResult {
        victory: lives > 0,
        lives_remaining: lives,
        max_lives,
        stars,
        gold_earned: total_gold_earned,
        gold_spent: total_gold_spent,
        waves_survived,
    }
}

// ---------------------------------------------------------------------------
// Printing helpers
// ---------------------------------------------------------------------------

fn print_header() {
    println!();
    println!(
        "{:<8} {:<8} {:<9} {:<6} {:<6} {:<8} {:<10} {:<10} {:<6}",
        "Level", "Strat", "Result", "Lives", "Stars", "Rating", "Earned", "Spent", "Waves"
    );
    println!("{}", "-".repeat(78));
}

fn print_result(level: u32, strategy: Strategy, r: &SimResult) {
    let result_str = if r.victory { "WIN" } else { "DEFEAT" };
    let lives_str = format!("{}/{}", r.lives_remaining, r.max_lives);
    println!(
        "{:<8} {:<8} {:<9} {:<6} {:<6} {:<8} {:<10} {:<10} {:<6}",
        level,
        strategy,
        result_str,
        lives_str,
        star_str(r.stars),
        r.stars,
        r.gold_earned,
        r.gold_spent,
        r.waves_survived
    );
}

// ---------------------------------------------------------------------------
// Tests — one per difficulty (loops all levels), plus granular per-level tests
// ---------------------------------------------------------------------------

#[test]
fn balance_easy_all_levels() {
    println!("\n=== BALANCE REPORT: Easy AI ===");
    print_header();
    let mut total_stars = 0u32;
    let mut wins = 0u32;
    for level in 1..=MAX_LEVELS {
        let r = simulate_level(level, Strategy::Easy);
        print_result(level, Strategy::Easy, &r);
        total_stars += r.stars;
        if r.victory {
            wins += 1;
        }
    }
    println!();
    println!(
        "Easy summary: {}/{} levels won, {}/{} total stars",
        wins,
        MAX_LEVELS,
        total_stars,
        MAX_LEVELS * 3
    );
    println!();
}

#[test]
fn balance_medium_all_levels() {
    println!("\n=== BALANCE REPORT: Medium AI ===");
    print_header();
    let mut total_stars = 0u32;
    let mut wins = 0u32;
    for level in 1..=MAX_LEVELS {
        let r = simulate_level(level, Strategy::Medium);
        print_result(level, Strategy::Medium, &r);
        total_stars += r.stars;
        if r.victory {
            wins += 1;
        }
    }
    println!();
    println!(
        "Medium summary: {}/{} levels won, {}/{} total stars",
        wins,
        MAX_LEVELS,
        total_stars,
        MAX_LEVELS * 3
    );
    println!();
}

#[test]
fn balance_hard_all_levels() {
    println!("\n=== BALANCE REPORT: Hard AI ===");
    print_header();
    let mut total_stars = 0u32;
    let mut wins = 0u32;
    for level in 1..=MAX_LEVELS {
        let r = simulate_level(level, Strategy::Hard);
        print_result(level, Strategy::Hard, &r);
        total_stars += r.stars;
        if r.victory {
            wins += 1;
        }
    }
    println!();
    println!(
        "Hard summary: {}/{} levels won, {}/{} total stars",
        wins,
        MAX_LEVELS,
        total_stars,
        MAX_LEVELS * 3
    );
    println!();
}

// ---------------------------------------------------------------------------
// Grand balance overview — all strategies side-by-side
// ---------------------------------------------------------------------------

#[test]
fn balance_overview() {
    println!("\n====================================================================");
    println!("          AGES OF AETHER — BALANCE SIMULATION OVERVIEW");
    println!("====================================================================");
    print_header();
    for level in 1..=MAX_LEVELS {
        for &strat in &[Strategy::Easy, Strategy::Medium, Strategy::Hard] {
            let r = simulate_level(level, strat);
            print_result(level, strat, &r);
        }
        println!();
    }

    // Summary table
    println!("=== SUMMARY ===");
    println!("{:<10} {:<8} {:<8} {:<8}", "Metric", "Easy", "Medium", "Hard");
    println!("{}", "-".repeat(38));

    let mut easy_wins = 0u32;
    let mut med_wins = 0u32;
    let mut hard_wins = 0u32;
    let mut easy_stars = 0u32;
    let mut med_stars = 0u32;
    let mut hard_stars = 0u32;

    for level in 1..=MAX_LEVELS {
        let e = simulate_level(level, Strategy::Easy);
        let m = simulate_level(level, Strategy::Medium);
        let h = simulate_level(level, Strategy::Hard);
        if e.victory { easy_wins += 1; }
        if m.victory { med_wins += 1; }
        if h.victory { hard_wins += 1; }
        easy_stars += e.stars;
        med_stars += m.stars;
        hard_stars += h.stars;
    }

    println!(
        "{:<10} {:<8} {:<8} {:<8}",
        "Wins",
        format!("{}/{}", easy_wins, MAX_LEVELS),
        format!("{}/{}", med_wins, MAX_LEVELS),
        format!("{}/{}", hard_wins, MAX_LEVELS),
    );
    println!(
        "{:<10} {:<8} {:<8} {:<8}",
        "Stars",
        format!("{}/{}", easy_stars, MAX_LEVELS * 3),
        format!("{}/{}", med_stars, MAX_LEVELS * 3),
        format!("{}/{}", hard_stars, MAX_LEVELS * 3),
    );
    println!();

    // Balance warnings
    println!("=== BALANCE WARNINGS ===");
    for level in 1..=MAX_LEVELS {
        let e = simulate_level(level, Strategy::Easy);
        let m = simulate_level(level, Strategy::Medium);
        let h = simulate_level(level, Strategy::Hard);

        if e.stars >= 3 {
            println!(
                "  [!] Level {} too easy: even Easy AI gets 3 stars",
                level
            );
        }
        if !h.victory {
            println!(
                "  [!] Level {} possibly too hard: Hard AI loses (survived {}/10 waves)",
                level, h.waves_survived
            );
        }
        if m.stars == 0 {
            println!(
                "  [!] Level {} harsh for medium: Medium AI gets 0 stars",
                level
            );
        }
    }
    println!();
}

// ---------------------------------------------------------------------------
// Individual per-level tests (useful for focused debugging)
// ---------------------------------------------------------------------------

macro_rules! level_test {
    ($name:ident, $level:expr) => {
        #[test]
        fn $name() {
            let info = level_info($level);
            println!(
                "\n--- Level {} \"{}\" ({}) ---",
                $level, info.name, info.era
            );
            let config = level_start_config($level);
            let spots = level_build_spots($level);
            let path = level_path($level);
            println!(
                "  Start gold: {}, Lives: {}, Build spots: {}, Path length: {:.1}",
                config.starting_gold,
                config.lives,
                spots.len(),
                path_length(&path)
            );
            println!(
                "  HP scale: {:.0}%/wave, Speed scale: {:.1}%/wave",
                config.wave_hp_scale * 100.0,
                config.wave_speed_scale * 100.0
            );

            // Print wave enemy counts
            let waves = level_waves($level);
            for (i, w) in waves.iter().enumerate() {
                let total_enemies: u32 = w.groups.iter().map(|g| g.count).sum();
                let types: Vec<String> = w
                    .groups
                    .iter()
                    .map(|g| format!("{:?}x{}", g.enemy_type, g.count))
                    .collect();
                println!("  Wave {}: {} enemies [{}]", i + 1, total_enemies, types.join(", "));
            }

            println!();
            for &strat in &[Strategy::Easy, Strategy::Medium, Strategy::Hard] {
                let r = simulate_level($level, strat);
                let result_str = if r.victory { "WIN" } else { "DEFEAT" };
                println!(
                    "  {:>6}: {} | Lives {}/{} | {} stars | Gold earned {} spent {} | Waves {}/10",
                    strat,
                    result_str,
                    r.lives_remaining,
                    r.max_lives,
                    r.stars,
                    r.gold_earned,
                    r.gold_spent,
                    r.waves_survived,
                );
            }
            println!();
        }
    };
}

level_test!(balance_level_01, 1);
level_test!(balance_level_02, 2);
level_test!(balance_level_03, 3);
level_test!(balance_level_04, 4);
level_test!(balance_level_05, 5);
level_test!(balance_level_06, 6);
level_test!(balance_level_07, 7);
level_test!(balance_level_08, 8);
level_test!(balance_level_09, 9);
level_test!(balance_level_10, 10);
