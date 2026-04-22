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
                // combat::enforce_opaque_enemies removed — was causing flicker via change detection
                golem::spawn_golem_visuals.in_set(GameSet::Spawning),
                golem::update_golem_visuals.in_set(GameSet::Visual),
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
                player_ability::animate_meteor_falling.in_set(GameSet::Visual),
                player_ability::tick_timed_despawns.in_set(GameSet::Cleanup),
                player_ability::update_targeting_ring.in_set(GameSet::Visual),
                combat::init_damage_tracking.in_set(GameSet::Spawning),
                combat::spawn_damage_numbers.in_set(GameSet::Visual),
                combat::tick_damage_numbers.in_set(GameSet::Visual),
                combat::animate_damage_numbers.in_set(GameSet::Visual),
                animate_placement_bounce.in_set(GameSet::Visual),
                animate_upgrade_flash.in_set(GameSet::Visual),
                animate_orbiting_indicators.in_set(GameSet::Visual),
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
                debug::admin_unlock_tap,
                debug::sync_admin_ui_visibility,
                debug::handle_admin_turn_off,
                debug::debug_hotkeys,
                debug::manage_debug_overlay,
                debug::update_debug_overlay,
                debug::handle_debug_buttons,
                apply_game_speed,
            )
                .run_if(in_state(AppState::Playing)),
        );

        // Focus-based audio pause runs in every state so minimizing always silences music.
        app.add_systems(Update, audio::pause_audio_on_focus);

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
/// Composite key: (level, specialized, spec_level) to detect changes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct IndicatorState(u8, bool, u8);

fn update_upgrade_indicators(
    mut commands: Commands,
    towers: Query<(
        Entity,
        &Transform,
        &crate::components::TowerLevel,
        &crate::components::Element,
        Option<&crate::components::TowerSpec>,
        Option<&crate::components::SpecLevel>,
    ), With<crate::components::Tower>>,
    existing: Query<(Entity, &crate::components::UpgradeIndicator)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tracked: Local<std::collections::HashMap<Entity, IndicatorState>>,
) {
    for (tower_entity, tower_transform, level, element, spec, spec_level) in &towers {
        let current = IndicatorState(
            level.0,
            spec.is_some(),
            spec_level.map(|s| s.0).unwrap_or(0),
        );
        let prev = tracked.get(&tower_entity).copied();
        if prev == Some(current) {
            continue;
        }
        tracked.insert(tower_entity, current);

        // Remove old indicators for this tower
        for (ind_entity, ind) in &existing {
            if ind.tower == tower_entity {
                commands.entity(ind_entity).despawn();
            }
        }

        let color = crate::data::element_color(*element);
        let emissive = crate::data::element_emissive(*element);
        let pos = tower_transform.translation;
        let indicator = crate::components::UpgradeIndicator { tower: tower_entity };
        let world = crate::components::GameWorldEntity;

        let glow_mat = materials.add(StandardMaterial {
            base_color: color,
            emissive,
            unlit: true,
            ..default()
        });
        let ring_mat = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.4),
            emissive: emissive * 0.4,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            ..default()
        });

        // --- Level 1: single upward cone (pennant) ---
        if level.0 == 1 {
            commands.spawn((
                Mesh3d(meshes.add(Cone { radius: 0.15, height: 0.5 })),
                MeshMaterial3d(glow_mat.clone()),
                Transform::from_translation(pos + Vec3::new(0.0, 3.2, 0.0)),
                indicator.clone(),
                world,
            ));
        }

        // --- Level 2 (not specialized): two cones + ground ring ---
        if level.0 >= 2 && !current.1 {
            // Twin cones flanking the tower
            for side in [-0.4_f32, 0.4] {
                commands.spawn((
                    Mesh3d(meshes.add(Cone { radius: 0.18, height: 0.6 })),
                    MeshMaterial3d(glow_mat.clone()),
                    Transform::from_translation(pos + Vec3::new(side, 3.4, 0.0)),
                    indicator.clone(),
                    world,
                ));
            }
            // Ground ring
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(1.1, 1.25))),
                MeshMaterial3d(ring_mat.clone()),
                Transform::from_translation(pos + Vec3::new(0.0, 0.05, 0.0))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                indicator.clone(),
                world,
            ));
        }

        // --- Specialized: element-specific crown shape + ground ring + orbiting particles ---
        if let Some(sl) = spec_level {
            // Ground ring (wider for specs)
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(1.3, 1.5))),
                MeshMaterial3d(ring_mat.clone()),
                Transform::from_translation(pos + Vec3::new(0.0, 0.05, 0.0))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                indicator.clone(),
                world,
            ));

            // Element-specific crown shape above tower
            use crate::components::Element as Elem;
            let crown_mesh: Mesh = match *element {
                Elem::Lightning => Cylinder::new(0.12, 0.8).into(),   // tall rod
                Elem::Earth => Cuboid::new(0.4, 0.4, 0.4).into(),    // solid cube
                Elem::Ice => Sphere::new(0.25).into(),                // crystal
                Elem::Fire => Cone { radius: 0.25, height: 0.7 }.into(), // flame point
            };
            let crown_scale = match *element {
                Elem::Ice => Vec3::new(0.8, 1.5, 0.8), // stretch into crystal
                _ => Vec3::ONE,
            };
            commands.spawn((
                Mesh3d(meshes.add(crown_mesh)),
                MeshMaterial3d(glow_mat.clone()),
                Transform::from_translation(pos + Vec3::new(0.0, 3.6, 0.0))
                    .with_scale(crown_scale),
                indicator.clone(),
                world,
            ));

            // Orbiting particles (1 per spec level)
            for i in 0..sl.0 {
                let angle = (i as f32 / sl.0 as f32) * std::f32::consts::TAU;
                let orbit_r = 1.5;
                let ox = angle.cos() * orbit_r;
                let oz = angle.sin() * orbit_r;
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.08))),
                    MeshMaterial3d(glow_mat.clone()),
                    Transform::from_translation(pos + Vec3::new(ox, 2.5, oz)),
                    indicator.clone(),
                    world,
                    crate::components::OrbitingIndicator {
                        center: pos,
                        radius: orbit_r,
                        height: 2.5,
                        speed: 1.5,
                        offset: angle,
                    },
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

/// Animate orbiting spec-level indicators around their tower.
fn animate_orbiting_indicators(
    mut orbiters: Query<(&mut Transform, &crate::components::OrbitingIndicator)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (mut transform, orbit) in &mut orbiters {
        let angle = orbit.offset + t * orbit.speed;
        transform.translation.x = orbit.center.x + angle.cos() * orbit.radius;
        transform.translation.z = orbit.center.z + angle.sin() * orbit.radius;
        transform.translation.y = orbit.center.y + orbit.height + (t * 2.0 + orbit.offset).sin() * 0.15;
    }
}
