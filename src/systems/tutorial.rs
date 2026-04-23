//! First-play tutorial (Level 1, first visit).
//!
//! Teaches the player three things:
//!   1. Tap a build spot → choose a tower (forced to Earth so we can teach step 3)
//!   2. Tap the tower → Set Rally Point → tap the ground
//!   3. Tap the Start Wave button
//!
//! The rally-point gesture (tap unit, tap destination) generalizes to hero
//! movement on later levels — no separate hero tutorial needed.
//!
//! Tutorial only runs when:
//!   - current level == 1
//!   - save.tutorial_completed == false
//! When it runs, it blocks non-Earth tower buttons and suppresses auto-wave,
//! then lets normal gameplay resume after the Start-Wave prompt.

use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    #[default]
    Inactive,
    /// Show "tap a build spot" hint. Advances when the build menu appears.
    TapBuildSpot,
    /// Build menu is open; show "tap Earth" hint. Advances when a tower spawns.
    BuildEarth,
    /// Earth tower exists; show "tap a golem" hint. Advances when SettingRallyPoint mode starts.
    TapGolem,
    /// Rally-point mode active; show "tap the path". Advances when rally point is set.
    TapGround,
    /// Rally set; show "tap Start Wave". Advances when wave 1 starts.
    TapStartWave,
    /// Done — tutorial completed successfully. Next Playing-enter clears.
    Completed,
}

#[derive(Resource, Default)]
pub struct TutorialState {
    pub step: TutorialStep,
    pub build_menu_seen: bool,
    pub built_tower: Option<Entity>,
    pub rally_set: bool,
}

#[derive(Component)]
pub struct TutorialOverlayRoot;

#[derive(Component)]
pub struct TutorialBanner;

// ---------------------------------------------------------------------------
// Activation / deactivation
// ---------------------------------------------------------------------------

pub fn activate_tutorial_on_enter(
    mut tutorial: ResMut<TutorialState>,
    current_level: Res<CurrentLevel>,
    save: Option<Res<crate::save::SaveData>>,
) {
    let should_run = current_level.0 == 1
        && save.as_ref().map(|s| !s.tutorial_completed).unwrap_or(true);
    tutorial.step = if should_run {
        TutorialStep::TapBuildSpot
    } else {
        TutorialStep::Inactive
    };
    tutorial.build_menu_seen = false;
    tutorial.built_tower = None;
    tutorial.rally_set = false;
}

pub fn is_tutorial_active(tutorial: &TutorialState) -> bool {
    !matches!(tutorial.step, TutorialStep::Inactive | TutorialStep::Completed)
}

// ---------------------------------------------------------------------------
// Step progression (drives off game state changes)
// ---------------------------------------------------------------------------

pub fn advance_tutorial(
    mut tutorial: ResMut<TutorialState>,
    selection: Res<Selection>,
    wave: Res<WaveState>,
    towers: Query<(Entity, &Element), With<Tower>>,
    mut save: Option<ResMut<crate::save::SaveData>>,
) {
    if !is_tutorial_active(&tutorial) {
        return;
    }

    match tutorial.step {
        TutorialStep::TapBuildSpot => {
            if matches!(*selection, Selection::BuildSpot(_)) {
                tutorial.step = TutorialStep::BuildEarth;
                tutorial.build_menu_seen = true;
            }
        }
        TutorialStep::BuildEarth => {
            if let Some((entity, _)) = towers.iter().find(|(_, e)| **e == Element::Earth) {
                tutorial.built_tower = Some(entity);
                tutorial.step = TutorialStep::TapGolem;
            }
        }
        TutorialStep::TapGolem => {
            if let (Selection::SettingRallyPoint(tower), Some(built)) =
                (&*selection, tutorial.built_tower)
            {
                if *tower == built {
                    tutorial.step = TutorialStep::TapGround;
                }
            }
        }
        TutorialStep::TapGround => {
            // When the SettingRallyPoint selection ends AND a TowerRallyPoint now exists
            // on the built tower, the player successfully placed the rally.
            if let Some(built) = tutorial.built_tower {
                let still_setting = matches!(*selection, Selection::SettingRallyPoint(e) if e == built);
                if !still_setting && tutorial.rally_set_detected(built) {
                    tutorial.step = TutorialStep::TapStartWave;
                }
            }
        }
        TutorialStep::TapStartWave => {
            // `game.wave_number` only increments after the wave finishes spawning,
            // so watch the wave phase instead — it leaves Idle the instant the wave
            // button press is consumed.
            if !matches!(wave.phase, WavePhase::Idle) {
                tutorial.step = TutorialStep::Completed;
                if let Some(save) = save.as_mut() {
                    if !save.tutorial_completed {
                        save.tutorial_completed = true;
                        crate::save::write_save(save);
                    }
                }
            }
        }
        _ => {}
    }
}

impl TutorialState {
    /// The rally-point check needs access to the TowerRallyPoint component, which
    /// is inconvenient to borrow inside `advance_tutorial`. We track a flag instead,
    /// flipped by `track_rally_placement` which runs earlier.
    fn rally_set_detected(&self, _built: Entity) -> bool {
        self.rally_set
    }
}

pub fn track_rally_placement(
    mut tutorial: ResMut<TutorialState>,
    towers: Query<Entity, (With<Tower>, With<TowerRallyPoint>)>,
) {
    if matches!(tutorial.step, TutorialStep::TapGround) {
        if let Some(built) = tutorial.built_tower {
            if towers.iter().any(|e| e == built) {
                tutorial.rally_set = true;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Overlay UI (banner at the top with current instruction)
// ---------------------------------------------------------------------------

fn instruction_text(step: TutorialStep) -> Option<&'static str> {
    match step {
        TutorialStep::TapBuildSpot => Some("Tap one of the gray build spots on the map."),
        TutorialStep::BuildEarth   => Some("Tap EARTH to build a golem barracks."),
        TutorialStep::TapGolem     => Some("Tap one of your golems."),
        TutorialStep::TapGround    => Some("Now tap the path to send your golem there."),
        TutorialStep::TapStartWave => Some("Tap the wave button to start the battle!"),
        _ => None,
    }
}

pub fn spawn_overlay(
    mut commands: Commands,
    tutorial: Res<TutorialState>,
    intro: Res<crate::systems::camera::CameraIntro>,
    existing: Query<Entity, With<TutorialOverlayRoot>>,
) {
    if !is_tutorial_active(&tutorial) {
        return;
    }
    // Wait for the zoom intro to finish so the banner doesn't compete with the camera move.
    if intro.active {
        return;
    }
    if !existing.is_empty() {
        return;
    }

    commands
        .spawn((
            TutorialOverlayRoot,
            GameWorldEntity,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        max_width: Val::Px(480.0),
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.05, 0.18, 0.92)),
                    BorderRadius::all(Val::Px(10.0)),
                    BorderColor(Color::srgb(1.0, 0.85, 0.4)),
                    Outline::new(Val::Px(2.0), Val::Px(0.0), Color::srgb(1.0, 0.85, 0.4)),
                ))
                .with_children(|card| {
                    card.spawn((
                        TutorialBanner,
                        Text::new(instruction_text(tutorial.step).unwrap_or("")),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.95, 0.7)),
                    ));
                });
        });
}

pub fn update_overlay_text(
    tutorial: Res<TutorialState>,
    mut banners: Query<&mut Text, With<TutorialBanner>>,
) {
    if !tutorial.is_changed() {
        return;
    }
    let Some(text) = instruction_text(tutorial.step) else { return };
    for mut t in &mut banners {
        t.0 = text.to_string();
    }
}

/// Despawn the overlay when the tutorial finishes (or was never active).
pub fn cleanup_overlay_on_complete(
    mut commands: Commands,
    tutorial: Res<TutorialState>,
    overlays: Query<Entity, With<TutorialOverlayRoot>>,
) {
    if matches!(tutorial.step, TutorialStep::Completed | TutorialStep::Inactive) {
        for e in &overlays {
            commands.entity(e).despawn_recursive();
        }
    }
}
