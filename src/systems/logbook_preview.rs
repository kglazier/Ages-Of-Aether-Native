use bevy::prelude::*;
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::camera::RenderTarget;
use std::collections::HashMap;

use crate::components::Element;
use crate::data::{self, EnemyType, ALL_ENEMY_TYPES};

/// Marker for logbook preview entities (models, cameras, lights).
#[derive(Component)]
pub struct LogbookPreviewEntity;

/// Marker for a preview model needing tint applied once its materials load.
#[derive(Component)]
pub struct LogbookPreviewTint(pub Color);

/// Marker for preview models that need their idle animation started.
#[derive(Component)]
pub struct LogbookPreviewNeedsAnim {
    pub idle_clip: Handle<AnimationClip>,
}


/// Resource storing rendered preview image handles.
#[derive(Resource, Default)]
pub struct LogbookPreviews {
    pub enemy_images: HashMap<String, Handle<Image>>,
    pub tower_images: HashMap<String, Handle<Image>>,
    pub spawned: bool,
}

const PREVIEW_SIZE: u32 = 128;
/// Spacing between preview model slots in world units.
const SLOT_SPACING: f32 = 12.0;
/// Y offset to push all preview models far below the game world.
const BASE_Y: f32 = -200.0;
/// Base angle for the 30-degree view camera orbit (radians).
const CAM_ANGLE: f32 = std::f32::consts::PI / 6.0;

/// Per-enemy preview scale. Uses in-game scale as base with adjustments
/// so each model fills the 128px thumbnail well.
fn enemy_preview_scale(enemy_type: EnemyType) -> f32 {
    let base = data::enemy_stats(enemy_type).model_scale;
    match enemy_type {
        // Blobs
        EnemyType::Amoeba | EnemyType::Sporebloom => base * 2.5,
        EnemyType::Jellyfish => base * 2.0,
        EnemyType::Trilobite => base * 2.0,
        EnemyType::SeaScorpion => base * 2.5,
        EnemyType::Nautilus => base * 1.5,
        EnemyType::GiantWorm => base * 1.0,
        // Dinos
        EnemyType::Raptor => base * 2.5,
        EnemyType::Parasaur | EnemyType::CompyHealer => base * 2.5,
        EnemyType::Stegosaurus => base * 1.1,
        EnemyType::Triceratops => base * 1.3,
        EnemyType::Pterodactyl | EnemyType::Wyvern => base * 2.0,
        EnemyType::TRex => base * 0.4,
        EnemyType::Dragon => base * 1.2,
        // Eagles
        EnemyType::GiantEagle => base * 1.6,
        EnemyType::EagleScout => base * 1.3,
        // Humanoids
        EnemyType::Caveman => base * 1.8,
        EnemyType::Footman => base * 1.8,
        EnemyType::Knight => base * 2.0,
        EnemyType::Shaman => base * 2.5,
        EnemyType::Medicus => base * 2.5,
        EnemyType::Priest => base * 2.5,
        EnemyType::Legionary => base * 2.5,
        EnemyType::Dodo => base * 2.5,
        EnemyType::Minotaur => base * 1.2,
        // Large animals
        EnemyType::Sabertooth | EnemyType::Lion => base * 1.5,
        EnemyType::Mammoth => base * 0.8,
        EnemyType::WoollyRhino => base * 0.7,
        EnemyType::WarElephant => base * 0.7,
        EnemyType::Cavalry => base * 1.8,
    }
}

fn tower_preview_scale(element: Element) -> f32 {
    let base = data::tower_stats(element, 0).model_scale;
    base * 1.2
}

fn create_preview_image(images: &mut Assets<Image>) -> Handle<Image> {
    let size = Extent3d {
        width: PREVIEW_SIZE,
        height: PREVIEW_SIZE,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    images.add(image)
}

/// Spawns all preview models and cameras. Called once when entering Logbook.
pub fn setup_logbook_previews(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut previews: ResMut<LogbookPreviews>,
) {
    if previews.spawned {
        return;
    }
    previews.spawned = true;

    let mut slot = 0usize;

    // Enemy previews
    for &enemy_type in &ALL_ENEMY_TYPES {
        let stats = data::enemy_stats(enemy_type);
        let info = data::enemy_info(enemy_type);
        let key = info.name.to_string();

        let image_handle = create_preview_image(&mut images);
        previews.enemy_images.insert(key, image_handle.clone());

        let x = (slot as f32) * SLOT_SPACING;
        let model_pos = Vec3::new(x, BASE_Y, 0.0);

        let scale = enemy_preview_scale(enemy_type);
        let mut transform = Transform::from_translation(model_pos)
            .with_scale(Vec3::splat(scale));

        // Apply any model-specific rotation from game data
        if stats.rotation_y != 0.0 {
            transform.rotate_y(stats.rotation_y);
        }

        // Per-model position/rotation fixes for preview
        match enemy_type {
            EnemyType::WoollyRhino => {
                // Model renders correctly with no rotation
            }
            EnemyType::TRex => {
                transform.translation.y -= 0.5;
            }
            EnemyType::Triceratops => {
                transform.translation.x += 0.5;
            }
            EnemyType::GiantEagle => {
                transform.translation.x += 1.0;
                transform.translation.y += 0.5;
            }
            EnemyType::EagleScout => {
                transform.translation.x += 1.0;
                transform.translation.y += 0.5;
            }
            EnemyType::Shaman | EnemyType::Caveman | EnemyType::Footman
            | EnemyType::Dragon => {
                transform.translation.y -= 0.8;
            }
            EnemyType::Minotaur => {
                transform.translation.y += 0.3;
            }
            EnemyType::Legionary => {
                transform.translation.y += 0.3;
            }
            EnemyType::Priest | EnemyType::Medicus => {
                transform.translation.y -= 0.8;
                // Rotate slightly so face is visible past hands in idle pose
                transform.rotate_y(0.4);
            }
            EnemyType::Cavalry => {
                transform.translation.y -= 0.5;
            }
            _ => {}
        }

        let model_path = stats.model_path;
        let scene = asset_server.load(format!("{}#Scene0", model_path));
        let mut model_cmds = commands.spawn((
            SceneRoot(scene),
            transform,
            LogbookPreviewEntity,
        ));

        if let Some([r, g, b]) = stats.tint {
            model_cmds.insert(LogbookPreviewTint(Color::srgb(r, g, b)));
        }

        // Load idle animation for humanoids with external Mixamo anims
        // Skip minotaur — its non-Mixamo skeleton distorts with shared Mixamo
        // idle clip; static rest pose looks correct.
        if let Some(anim_files) = stats.anim_files {
            if enemy_type != EnemyType::Minotaur {
                let idle_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", anim_files[1]));
                model_cmds.insert(LogbookPreviewNeedsAnim { idle_clip });
            }
        } else if stats.anim_indices != [255; 4] {
            // Embedded animation — load idle index
            let idle_idx = stats.anim_indices[1];
            let idle_clip = asset_server.load(
                format!("{}#Animation{}", stats.model_path, idle_idx)
            );
            model_cmds.insert(LogbookPreviewNeedsAnim { idle_clip });
        }

        // Cavalry: mount the knight model on the horse
        if enemy_type == EnemyType::Cavalry {
            let knight_scene = asset_server.load("models/enemies/cavalry-knight.glb#Scene0");
            model_cmds.with_child((
                SceneRoot(knight_scene),
                Transform::from_translation(Vec3::new(0.0, 1.0, 0.0))
                    .with_scale(Vec3::splat(0.013)),
            ));
        }

        // Key light — front-right
        commands.spawn((
            PointLight {
                intensity: 2_000_000.0,
                range: 25.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(model_pos + Vec3::new(3.0, 5.0, 5.0)),
            LogbookPreviewEntity,
        ));
        // Fill light — front-left, softer
        commands.spawn((
            PointLight {
                intensity: 1_000_000.0,
                range: 25.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(model_pos + Vec3::new(-4.0, 3.0, 4.0)),
            LogbookPreviewEntity,
        ));

        // Camera at 45-degree angle — orbit around Y axis
        let cam_dist = 5.0;
        let cam_height = 1.5;
        let cam_pos = model_pos + Vec3::new(
            cam_dist * CAM_ANGLE.sin(),
            cam_height,
            cam_dist * CAM_ANGLE.cos(),
        );
        let look_at = model_pos + Vec3::new(0.0, 0.8, 0.0);
        commands.spawn((
            Camera3d::default(),
            Camera {
                target: RenderTarget::Image(image_handle),
                clear_color: ClearColorConfig::Custom(Color::srgba(0.08, 0.05, 0.12, 1.0)),
                order: -10 - (slot as isize),
                ..default()
            },
            Transform::from_translation(cam_pos).looking_at(look_at, Vec3::Y),
            Msaa::Off,
            LogbookPreviewEntity,
        ));

        slot += 1;
    }

    // Tower previews
    let elements = [Element::Lightning, Element::Earth, Element::Ice, Element::Fire];
    for &element in &elements {
        let base = data::tower_stats(element, 0);
        let key = base.name.to_string();

        let image_handle = create_preview_image(&mut images);
        previews.tower_images.insert(key, image_handle.clone());

        let x = (slot as f32) * SLOT_SPACING;
        let model_pos = Vec3::new(x, BASE_Y, 0.0);

        let scale = tower_preview_scale(element);
        let scene = asset_server.load(format!("{}#Scene0", base.model_path));
        commands.spawn((
            SceneRoot(scene),
            Transform::from_translation(model_pos).with_scale(Vec3::splat(scale)),
            LogbookPreviewEntity,
        ));

        // Key light
        commands.spawn((
            PointLight {
                intensity: 2_000_000.0,
                range: 25.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(model_pos + Vec3::new(3.0, 5.0, 5.0)),
            LogbookPreviewEntity,
        ));
        // Fill light
        commands.spawn((
            PointLight {
                intensity: 1_000_000.0,
                range: 25.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(model_pos + Vec3::new(-4.0, 3.0, 4.0)),
            LogbookPreviewEntity,
        ));

        // Camera at 45-degree angle
        let cam_dist = 5.0;
        let cam_height = 2.0;
        let cam_pos = model_pos + Vec3::new(
            cam_dist * CAM_ANGLE.sin(),
            cam_height,
            cam_dist * CAM_ANGLE.cos(),
        );
        let look_at = model_pos + Vec3::new(0.0, 1.0, 0.0);
        commands.spawn((
            Camera3d::default(),
            Camera {
                target: RenderTarget::Image(image_handle),
                clear_color: ClearColorConfig::Custom(Color::srgba(0.08, 0.05, 0.12, 1.0)),
                order: -10 - (slot as isize),
                ..default()
            },
            Transform::from_translation(cam_pos).looking_at(look_at, Vec3::Y),
            Msaa::Off,
            LogbookPreviewEntity,
        ));

        slot += 1;
    }
}

/// Applies tint to preview models once their materials are loaded.
/// Uses the same HSL recoloring as the in-game enemy tint system.
pub fn apply_preview_tints(
    mut commands: Commands,
    tint_q: Query<(Entity, &LogbookPreviewTint, &Children)>,
    children_q: Query<&Children>,
    mesh_q: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, tint, children) in &tint_q {
        let tint_srgba = tint.0.to_srgba();
        let species_hue = rgb_to_hsl(tint_srgba.red, tint_srgba.green, tint_srgba.blue);

        // Collect all mesh parts in hierarchy
        struct MeshPart {
            entity: Entity,
            handle: Handle<StandardMaterial>,
            lightness: f32,
        }
        let mut parts: Vec<MeshPart> = Vec::new();
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(child) = stack.pop() {
            if let Ok((mesh_entity, mat_handle)) = mesh_q.get(child) {
                if let Some(original) = materials.get(&mat_handle.0) {
                    let orig = original.base_color.to_srgba();
                    let (_, _, l) = rgb_to_hsl(orig.red, orig.green, orig.blue);
                    parts.push(MeshPart {
                        entity: mesh_entity,
                        handle: mat_handle.0.clone(),
                        lightness: l,
                    });
                }
            }
            if let Ok(gc) = children_q.get(child) {
                stack.extend(gc.iter());
            }
        }

        if parts.is_empty() {
            continue; // Scene not loaded yet
        }

        // Normalize lightness and apply species hue (same as in-game)
        let min_l = parts.iter().map(|p| p.lightness).fold(f32::INFINITY, f32::min);
        let max_l = parts.iter().map(|p| p.lightness).fold(f32::NEG_INFINITY, f32::max);
        let range = (max_l - min_l).max(0.001);

        for (i, part) in parts.iter().enumerate() {
            if let Some(original) = materials.get(&part.handle) {
                let mut new_mat = original.clone();
                let t = if parts.len() <= 1 {
                    0.5
                } else if range < 0.01 {
                    i as f32 / (parts.len() as f32 - 1.0).max(1.0)
                } else {
                    ((part.lightness - min_l) / range).clamp(0.0, 1.0)
                };

                let target_l = 0.32 + t * 0.44;
                let hue_shift = (t - 0.5) * 0.06;
                let h = species_hue.0 + hue_shift;
                let s = species_hue.1 * 0.85;

                let (r, g, b) = hsl_to_rgb(h, s, target_l);
                new_mat.base_color = Color::srgb(r, g, b);
                new_mat.base_color_texture = None;
                new_mat.perceptual_roughness = 1.0;
                new_mat.metallic = 0.0;
                new_mat.alpha_mode = AlphaMode::Opaque;
                new_mat.emissive = LinearRgba::new(r * 0.15, g * 0.15, b * 0.15, 1.0);

                let new_handle = materials.add(new_mat);
                commands.entity(part.entity).insert(MeshMaterial3d(new_handle));
            }
        }

        commands.entity(entity).remove::<LogbookPreviewTint>();
    }
}

// HSL utilities (same as combat.rs)
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 0.001 { return (0.0, 0.0, l); }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 0.001 {
        let mut h = (g - b) / d;
        if g < b { h += 6.0; }
        h / 6.0
    } else if (max - g).abs() < 0.001 {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 0.001 { return (l, l, l); }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    (hue_to_rgb(p, q, h + 1.0 / 3.0), hue_to_rgb(p, q, h), hue_to_rgb(p, q, h - 1.0 / 3.0))
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

/// Starts idle animations on preview models once their AnimationPlayer is ready.
/// If the model has no embedded AnimationPlayer (e.g. minotaur-mixamo), manually
/// inserts one on the Armature node with AnimationTargets on all bones — matching
/// the in-game setup_enemy_animations and showcase_setup_anims approach.
pub fn start_preview_anims(
    mut commands: Commands,
    preview_q: Query<(Entity, &LogbookPreviewNeedsAnim, &Children)>,
    children_q: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    names: Query<&Name>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (entity, needs_anim, children) in &preview_q {
        // Walk hierarchy to find existing AnimationPlayer or Armature node
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        let mut found_player = None;
        let mut armature_entity = None;

        while let Some(child) = stack.pop() {
            if players.get(child).is_ok() {
                found_player = Some(child);
                break;
            }
            if let Ok(name) = names.get(child) {
                let n = name.as_str();
                if n == "Armature" || n.contains("CharacterArmature") {
                    armature_entity = Some(child);
                }
            }
            if let Ok(gc) = children_q.get(child) {
                stack.extend(gc.iter().copied());
            }
        }

        // If no AnimationPlayer found, manually create one on the Armature
        let player_entity = if let Some(pe) = found_player {
            pe
        } else if let Some(armature) = armature_entity {
            commands.entity(armature).insert(AnimationPlayer::default());
            // Insert AnimationTarget on armature and all bones
            let armature_name = names.get(armature)
                .map(|n| n.clone())
                .unwrap_or_else(|_| Name::new("Armature"));
            let root_path = vec![armature_name];
            commands.entity(armature).insert(AnimationTarget {
                id: AnimationTargetId::from_names(root_path.iter()),
                player: armature,
            });
            if let Ok(arm_children) = children_q.get(armature) {
                for &child in arm_children.iter() {
                    insert_preview_anim_targets(
                        &mut commands, child, armature,
                        &root_path, &children_q, &names,
                    );
                }
            }
            armature
        } else {
            continue; // Scene not loaded yet
        };

        if let Some(mut player) = found_player.and_then(|pe| players.get_mut(pe).ok()) {
            // Existing player — set up graph and play immediately
            let mut graph = AnimationGraph::new();
            let node = graph.add_clip(needs_anim.idle_clip.clone(), 1.0, graph.root);
            let graph_handle = graphs.add(graph);
            commands.entity(player_entity).insert((
                AnimationGraphHandle(graph_handle),
                AnimationTransitions::new(),
            ));
            player.play(node).repeat();
            commands.entity(entity).remove::<LogbookPreviewNeedsAnim>();
        }
        // else: AnimationPlayer was just inserted via commands — wait for next
        // frame so it exists and we can call play().
    }
}

/// Recursively insert AnimationTarget on bone entities for preview models.
fn insert_preview_anim_targets(
    commands: &mut Commands,
    entity: Entity,
    player: Entity,
    parent_path: &[Name],
    children_q: &Query<&Children>,
    names: &Query<&Name>,
) {
    let name = names.get(entity)
        .map(|n| n.clone())
        .unwrap_or_else(|_| Name::new(format!("bone_{}", entity.index())));
    let mut path = parent_path.to_vec();
    path.push(name);
    let id = AnimationTargetId::from_names(path.iter());
    commands.entity(entity).insert(AnimationTarget { id, player });
    if let Ok(children) = children_q.get(entity) {
        for &child in children.iter() {
            insert_preview_anim_targets(commands, child, player, &path, children_q, names);
        }
    }
}

/// Cleans up all preview entities when leaving the logbook.
pub fn cleanup_logbook_previews(
    mut commands: Commands,
    entities: Query<Entity, With<LogbookPreviewEntity>>,
    mut previews: ResMut<LogbookPreviews>,
) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
    previews.spawned = false;
    previews.enemy_images.clear();
    previews.tower_images.clear();
}
