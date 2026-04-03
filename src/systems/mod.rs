pub mod anim_test;
pub mod audio;
pub mod camera;
mod combat;
pub mod debug;
mod enemy_anim;
mod game_over;
pub mod hero_ability;
mod golem;
mod hero;
mod input;
pub mod logbook_preview;
mod path;
pub mod player_ability;
pub mod setup;
pub mod showcase;
pub mod tower_spec;
mod wave;

pub use camera::CameraFocus;

use bevy::prelude::*;
use crate::states::AppState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Spawning.after(GameSet::Input),
                GameSet::Movement.after(GameSet::Spawning),
                GameSet::Combat.after(GameSet::Movement),
                GameSet::Cleanup.after(GameSet::Combat),
                GameSet::Visual.after(GameSet::Cleanup),
            )
                .run_if(in_state(AppState::Playing)),
        );

        // Model showcase state
        app.add_systems(OnEnter(AppState::ModelShowcase), showcase::setup_showcase);
        app.add_systems(
            Update,
            (
                showcase::showcase_setup_anims,
                showcase::showcase_play_anims,
                showcase::showcase_camera,
            ).run_if(in_state(AppState::ModelShowcase)),
        );

        // Model debug state (Northern Outsider variants)
        app.add_systems(OnEnter(AppState::ModelDebug), showcase::setup_model_debug);
        app.add_systems(
            Update,
            (
                showcase::showcase_setup_anims,
                showcase::showcase_play_anims,
                showcase::showcase_camera,
                showcase::strip_debug_rotation_only_clips,
                showcase::debug_back_button,
                showcase::debug_mover_tick,
                showcase::cavalry_debug_tick,
            ).run_if(in_state(AppState::ModelDebug)),
        );
        app.add_systems(OnExit(AppState::ModelDebug), showcase::cleanup_model_debug);

        // Hero select — reuse showcase anim systems for hero preview
        app.add_systems(
            Update,
            (
                showcase::showcase_setup_anims,
                showcase::showcase_play_anims,
                showcase::strip_debug_rotation_only_clips,
            ).run_if(in_state(AppState::HeroSelect)),
        );

        // Animation test state
        app.add_systems(OnEnter(AppState::AnimTest), anim_test::setup_anim_test);
        app.add_systems(
            Update,
            (
                anim_test::anim_test_setup_skeleton,
                anim_test::anim_test_start_idle,
                anim_test::anim_test_ref_setup,
                anim_test::anim_test_ref_play,
                anim_test::anim_test_input,
                anim_test::anim_test_camera,
                anim_test::anim_test_log_ref_hierarchy,
            ).run_if(in_state(AppState::AnimTest)),
        );

        // Wait for native window before spawning 3D entities.
        // On Android, the render surface is null until Event::Resumed.
        app.add_systems(
            Update,
            setup::wait_for_window.run_if(in_state(AppState::WaitingForWindow)),
        );

        // Audio loading doesn't need the render surface — load early
        app.add_systems(Startup, audio::load_audio);

        // Scene setup runs when window is confirmed ready.
        // Only runs when NeedsFreshSetup is true (not on resume from pause).
        app.add_systems(OnEnter(AppState::Playing), (
            setup::cleanup_game_world
                .run_if(|needs: Res<crate::resources::NeedsFreshSetup>| needs.0),
            setup::setup_level
                .after(setup::cleanup_game_world)
                .run_if(|needs: Res<crate::resources::NeedsFreshSetup>| needs.0),
            hero::spawn_hero
                .after(setup::setup_level)
                .run_if(|needs: Res<crate::resources::NeedsFreshSetup>| needs.0),
            clear_fresh_setup.after(hero::spawn_hero),
        ));

        app.add_systems(
            Update,
            (
                camera::camera_control.in_set(GameSet::Input),
                input::handle_world_click.in_set(GameSet::Input),
                wave::auto_wave_tick.in_set(GameSet::Input),
                wave::wave_spawner.in_set(GameSet::Spawning),
                wave::heal_hero_on_wave_start.in_set(GameSet::Spawning),
                golem::spawn_golems.in_set(GameSet::Spawning),
                golem::setup_golem_animations.in_set(GameSet::Spawning),
                golem::play_golem_animations.in_set(GameSet::Spawning),
                path::move_enemies.in_set(GameSet::Movement),
                golem::golem_movement.in_set(GameSet::Movement),
                golem::update_golem_animations.in_set(GameSet::Movement),
                golem::golem_assign_targets.in_set(GameSet::Input),
                hero::block_enemies.in_set(GameSet::Input),
                golem::golem_melee_attack.in_set(GameSet::Combat),
                golem::enemies_attack_golem.in_set(GameSet::Combat),
                enemy_anim::setup_enemy_animations.in_set(GameSet::Spawning),
                enemy_anim::play_enemy_walk_anim.in_set(GameSet::Spawning),
                enemy_anim::update_enemy_animations.in_set(GameSet::Movement),
                enemy_anim::animate_procedural_walk.in_set(GameSet::Movement),
                enemy_anim::tick_dying_enemies.in_set(GameSet::Cleanup),
            ),
        );
        // Enemy animation helpers (separate call due to Bevy tuple size limit)
        app.add_systems(
            Update,
            (
                golem::strip_golem_root_motion.in_set(GameSet::Spawning),
                enemy_anim::discover_leg_bones.in_set(GameSet::Spawning),
                enemy_anim::strip_enemy_clip_root_motion.in_set(GameSet::Spawning),
                enemy_anim::animate_cavalry_knight.in_set(GameSet::Movement),
                enemy_anim::rock_single_clip_enemies.in_set(GameSet::Visual),
                wave::apply_enemy_model_rotation.in_set(GameSet::Spawning),
                combat::fix_blend_enemy_materials.in_set(GameSet::Spawning),
            ),
        );
        // Hero systems (separate call due to Bevy tuple size limit)
        app.add_systems(
            Update,
            (
                hero::hero_consume_move_command.in_set(GameSet::Input),
                hero::hero_movement.in_set(GameSet::Movement),
                hero::hero_auto_attack.in_set(GameSet::Combat),
                hero::enemies_attack_hero.in_set(GameSet::Combat),
                hero::hero_death_check.in_set(GameSet::Cleanup),
                hero::hero_respawn_tick.in_set(GameSet::Cleanup),
                hero::spawn_hero_visuals.in_set(GameSet::Spawning),
                hero::apply_hero_model_offset.in_set(GameSet::Spawning),
                hero::setup_hero_animations.in_set(GameSet::Spawning),
                hero::strip_hero_root_motion_clips.in_set(GameSet::Spawning),
                hero::strip_hero_rotation_only_clips.in_set(GameSet::Spawning),
                hero::play_hero_animations.in_set(GameSet::Spawning),
                hero::update_hero_animations.in_set(GameSet::Movement),
                hero::hero_passive_regen.in_set(GameSet::Combat),
                hero::update_hero_visuals.in_set(GameSet::Visual),
                hero::update_hero_move_marker.in_set(GameSet::Visual),
                hero_ability::tick_ability_cooldowns.in_set(GameSet::Combat),
                hero_ability::tick_hero_buffs.in_set(GameSet::Combat),
                hero_ability::execute_ability.in_set(GameSet::Combat),
            ),
        );
        app.add_systems(
            Update,
            (
                combat::tower_targeting.in_set(GameSet::Combat),
                combat::move_projectiles.in_set(GameSet::Combat),
                combat::animate_muzzle_flashes.in_set(GameSet::Combat),
                combat::tick_debuffs.in_set(GameSet::Combat),
                combat::healer_aura_tick.in_set(GameSet::Combat),
                combat::golem_elemental_synergy.in_set(GameSet::Combat),
                combat::update_range_indicator.in_set(GameSet::Input),
                combat::spawn_health_bars.in_set(GameSet::Spawning),
                combat::check_enemy_death.in_set(GameSet::Cleanup),
                combat::animate_death_effects.in_set(GameSet::Cleanup),
                combat::animate_gold_popups.in_set(GameSet::Cleanup),
                combat::check_enemy_leak.in_set(GameSet::Cleanup),
                combat::apply_enemy_tints.in_set(GameSet::Cleanup),
                combat::update_health_bars.in_set(GameSet::Visual),
                golem::fix_golem_materials.in_set(GameSet::Cleanup),
                golem::check_golem_death.in_set(GameSet::Cleanup),
                golem::cleanup_orphan_golems.in_set(GameSet::Cleanup),
                position_towers_on_spots.in_set(GameSet::Cleanup),
                update_upgrade_indicators.in_set(GameSet::Cleanup),
                game_over::check_game_over.in_set(GameSet::Cleanup),
            ),
        );
        // Healer rings + ground mesh stripping & tower specializations
        app.add_systems(
            Update,
            combat::update_healer_rings.in_set(GameSet::Visual)
                .run_if(in_state(AppState::Playing)),
        );
        app.add_systems(
            Update,
            (
                combat::hide_ground_meshes.in_set(GameSet::Cleanup),
                tower_spec::apply_specialization.in_set(GameSet::Combat),
                tower_spec::apply_spec_upgrade.in_set(GameSet::Combat),
                tower_spec::tick_tower_auras.in_set(GameSet::Combat),
                tower_spec::tick_burn_zones.in_set(GameSet::Combat),
                player_ability::tick_player_ability_cooldowns.in_set(GameSet::Combat),
                player_ability::execute_player_ability.in_set(GameSet::Combat),
                player_ability::tick_reinforcements.in_set(GameSet::Combat),
                combat::init_damage_tracking.in_set(GameSet::Spawning),
                combat::spawn_damage_numbers.in_set(GameSet::Visual),
                combat::tick_damage_numbers.in_set(GameSet::Visual),
                combat::animate_damage_numbers.in_set(GameSet::Visual),
                animate_placement_bounce.in_set(GameSet::Visual),
                animate_upgrade_flash.in_set(GameSet::Visual),
                setup::animate_lava.in_set(GameSet::Visual),
            ),
        );
        app.add_systems(
            Update,
            (
                audio::check_audio_loaded,
                audio::start_battle_music,
                audio::play_death_sfx,
                audio::play_tower_attack_sfx,
                audio::play_wave_sfx,
                audio::pause_audio_on_focus,
                debug::debug_hotkeys,
                debug::manage_debug_overlay,
                debug::update_debug_overlay,
                debug::handle_debug_buttons,
                apply_game_speed,
            )
                .run_if(in_state(AppState::Playing)),
        );

        // Music volume sync runs in both Playing and Paused so slider changes take effect immediately
        app.add_systems(
            Update,
            audio::sync_music_volume
                .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
        );

        // Root motion cancellation — run AFTER animate_targets applies bone transforms
        // but BEFORE TransformPropagate computes GlobalTransform for rendering.
        app.add_systems(
            PostUpdate,
            hero::cancel_hero_root_motion
                .after(bevy::animation::animate_targets)
                .before(bevy::transform::TransformSystem::TransformPropagate)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Clears the NeedsFreshSetup flag after level setup completes.
fn clear_fresh_setup(mut needs: ResMut<crate::resources::NeedsFreshSetup>) {
    needs.0 = false;
}

/// Applies the GameSpeed resource to Bevy's virtual time scale.
fn apply_game_speed(
    speed: Res<crate::resources::GameSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    if time.relative_speed() != speed.0 {
        time.set_relative_speed(speed.0);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Spawning,
    Movement,
    Combat,
    Cleanup,
    Visual,
}

/// Spawns small star-like indicators above upgraded towers.
fn update_upgrade_indicators(
    mut commands: Commands,
    towers: Query<(Entity, &Transform, &crate::components::TowerLevel, &crate::components::Element), With<crate::components::Tower>>,
    existing: Query<(Entity, &crate::components::UpgradeIndicator)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tracked: Local<std::collections::HashMap<Entity, u8>>,
) {
    for (tower_entity, tower_transform, level, element) in &towers {
        let prev = tracked.get(&tower_entity).copied().unwrap_or(255);
        if prev == level.0 {
            continue;
        }
        tracked.insert(tower_entity, level.0);

        // Remove old indicators for this tower
        for (ind_entity, ind) in &existing {
            if ind.tower == tower_entity {
                commands.entity(ind_entity).despawn();
            }
        }

        // Spawn level indicators (small spheres) — 1 per level above 0
        if level.0 > 0 {
            let color = crate::data::element_color(*element);
            let emissive = crate::data::element_emissive(*element);
            for i in 0..level.0 {
                let offset_x = (i as f32 - (level.0 as f32 - 1.0) / 2.0) * 0.4;
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.12))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        emissive,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(
                        tower_transform.translation + Vec3::new(offset_x, 3.0, 0.0),
                    ),
                    crate::components::UpgradeIndicator { tower: tower_entity },
                    crate::components::GameWorldEntity,
                ));
            }
        }
    }
}

/// Positions newly spawned towers at their build spot location.
fn position_towers_on_spots(
    mut towers: Query<
        (&mut Transform, &crate::components::BuildSpotRef),
        Added<crate::components::Tower>,
    >,
    spots: Query<
        &Transform,
        (
            With<crate::components::BuildSpot>,
            Without<crate::components::Tower>,
        ),
    >,
) {
    for (mut tower_transform, spot_ref) in &mut towers {
        if let Ok(spot_transform) = spots.get(spot_ref.0) {
            tower_transform.translation = spot_transform.translation;
        }
    }
}

/// Animates tower placement with easeOutBack bounce (scale 0 → target with overshoot).
fn animate_placement_bounce(
    mut commands: Commands,
    mut towers: Query<(Entity, &mut Transform, &mut crate::components::PlacementBounce)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut bounce) in &mut towers {
        bounce.elapsed += dt;
        let t = (bounce.elapsed / bounce.duration).min(1.0);

        // easeOutBack: overshoot then settle
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        let ease = 1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2);

        let scale = bounce.target_scale * ease;
        transform.scale = Vec3::splat(scale);

        if t >= 1.0 {
            transform.scale = Vec3::splat(bounce.target_scale);
            commands.entity(entity).remove::<crate::components::PlacementBounce>();
        }
    }
}

/// Brief emissive flash on tower after upgrade — walks mesh hierarchy to spike emissive.
fn animate_upgrade_flash(
    mut commands: Commands,
    mut towers: Query<(Entity, &Children, &mut crate::components::UpgradeFlash)>,
    children_q: Query<&Children>,
    mesh_q: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, children, mut flash) in &mut towers {
        flash.remaining -= dt;

        // Flash intensity — bright at start, fading
        let intensity = (flash.remaining / 0.3).max(0.0);
        let emissive_boost = LinearRgba::new(intensity * 3.0, intensity * 3.0, intensity * 2.0, 1.0);

        // Walk hierarchy to find meshes
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(child) = stack.pop() {
            if let Ok(mat_handle) = mesh_q.get(child) {
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.emissive = emissive_boost;
                }
            }
            if let Ok(grandchildren) = children_q.get(child) {
                stack.extend(grandchildren.iter());
            }
        }

        if flash.remaining <= 0.0 {
            // Reset emissive to zero
            let mut stack: Vec<Entity> = children.iter().copied().collect();
            while let Some(child) = stack.pop() {
                if let Ok(mat_handle) = mesh_q.get(child) {
                    if let Some(mat) = materials.get_mut(&mat_handle.0) {
                        mat.emissive = LinearRgba::NONE;
                    }
                }
                if let Ok(grandchildren) = children_q.get(child) {
                    stack.extend(grandchildren.iter());
                }
            }
            commands.entity(entity).remove::<crate::components::UpgradeFlash>();
        }
    }
}
