use bevy::prelude::*;
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

/// Preview-specific scale overrides for models that are too big/small at their game scale.
fn enemy_preview_scale(enemy_type: EnemyType) -> f32 {
    let stats = data::enemy_stats(enemy_type);
    let base = stats.model_scale;
    // Normalize so models fill roughly the same viewport area
    match enemy_type {
        // Blobs are small at 0.3-0.7 scale; bump them up
        EnemyType::Amoeba | EnemyType::Jellyfish | EnemyType::Sporebloom => base * 3.0,
        EnemyType::Trilobite => base * 2.5,
        EnemyType::SeaScorpion | EnemyType::Nautilus => base * 3.5,
        EnemyType::GiantWorm => base * 2.0,
        // Dinos at 0.12-0.35 need a boost
        EnemyType::Raptor | EnemyType::Parasaur | EnemyType::CompyHealer => base * 7.0,
        EnemyType::Stegosaurus | EnemyType::Triceratops => base * 5.5,
        EnemyType::Pterodactyl | EnemyType::Wyvern => base * 3.5,
        EnemyType::TRex => base * 3.0,
        EnemyType::Dragon => base * 1.8,
        // Eagles are very small scale
        EnemyType::GiantEagle | EnemyType::EagleScout => base * 45.0,
        // Legionary is tiny
        EnemyType::Legionary => base * 45.0,
        // Dodo is small
        EnemyType::Dodo => base * 8.0,
        // Humanoids at ~0.5-1.0 are fine with small boost
        EnemyType::Caveman | EnemyType::Footman | EnemyType::Knight => base * 1.2,
        EnemyType::Shaman | EnemyType::Medicus | EnemyType::Priest => base * 1.5,
        EnemyType::Minotaur => base * 2.2,
        // Animals at 1.0-2.5
        EnemyType::Sabertooth | EnemyType::Lion => base * 1.0,
        EnemyType::Mammoth | EnemyType::WoollyRhino | EnemyType::WarElephant => base * 0.6,
        EnemyType::Cavalry => base * 2.2,
    }
}

fn tower_preview_scale(element: Element) -> f32 {
    match element {
        Element::Lightning => 3.5,
        Element::Earth => 1.0,
        Element::Ice => 1.2,
        Element::Fire => 1.8,
    }
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

    // Ambient light for previews (all share the same ambient)
    // The game's Playing state sets its own ambient, so this is fine for menu states.

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
        if stats.rotation_y != 0.0 {
            transform.rotate_y(stats.rotation_y);
        }

        let scene = asset_server.load(format!("{}#Scene0", stats.model_path));
        let mut model_cmds = commands.spawn((
            SceneRoot(scene),
            transform,
            LogbookPreviewEntity,
        ));

        if let Some([r, g, b]) = stats.tint {
            model_cmds.insert(LogbookPreviewTint(Color::srgb(r, g, b)));
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

        // Point light for this model
        commands.spawn((
            PointLight {
                intensity: 800_000.0,
                range: 20.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(model_pos + Vec3::new(3.0, 5.0, 5.0)),
            LogbookPreviewEntity,
        ));

        // Camera rendering to image
        let cam_pos = model_pos + Vec3::new(0.0, 1.5, 5.0);
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

        // Light
        commands.spawn((
            PointLight {
                intensity: 800_000.0,
                range: 20.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(model_pos + Vec3::new(3.0, 5.0, 5.0)),
            LogbookPreviewEntity,
        ));

        // Camera
        let cam_pos = model_pos + Vec3::new(0.0, 2.0, 5.0);
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
pub fn apply_preview_tints(
    mut commands: Commands,
    tint_q: Query<(Entity, &LogbookPreviewTint, &Children)>,
    children_q: Query<&Children>,
    mesh_q: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, tint, children) in &tint_q {
        let mut found = false;
        // Walk the hierarchy to find mesh materials
        for descendant in children.iter() {
            // Check direct children and their children
            if let Ok(mat_handle) = mesh_q.get(*descendant) {
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.base_color = tint.0;
                    found = true;
                }
            }
            if let Ok(grandchildren) = children_q.get(*descendant) {
                for gc in grandchildren.iter() {
                    if let Ok(mat_handle) = mesh_q.get(*gc) {
                        if let Some(mat) = materials.get_mut(&mat_handle.0) {
                            mat.base_color = tint.0;
                            found = true;
                        }
                    }
                    // One more level for deeply nested scenes
                    if let Ok(ggchildren) = children_q.get(*gc) {
                        for ggc in ggchildren.iter() {
                            if let Ok(mat_handle) = mesh_q.get(*ggc) {
                                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                                    mat.base_color = tint.0;
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if found {
            commands.entity(entity).remove::<LogbookPreviewTint>();
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
