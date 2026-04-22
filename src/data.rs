use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Level 1 — Primordial Pools
// ---------------------------------------------------------------------------

pub fn level1_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 0.0),
        Vec3::new(-9.0, 0.0, 0.0),
        Vec3::new(-7.0, 0.0, -6.0),
        Vec3::new(-3.0, 0.0, -7.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 7.0),
        Vec3::new(7.0, 0.0, 7.0),
        Vec3::new(9.0, 0.0, 0.0),
        Vec3::new(18.0, 0.0, 0.0),
    ]
}

pub fn level1_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-8.0, 0.0, 3.0),
        Vec3::new(-10.0, 0.0, -2.5),
        Vec3::new(-4.0, 0.0, -9.0),
        Vec3::new(-1.0, 0.0, 4.0),
        Vec3::new(1.0, 0.0, -4.0),
        Vec3::new(4.0, 0.0, 9.5),
        Vec3::new(10.0, 0.0, 3.0),
        Vec3::new(8.0, 0.0, -4.0),
    ]
}

// ---------------------------------------------------------------------------
// Enemy types & stats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyType {
    // Primordial
    Amoeba,
    Jellyfish,
    Sporebloom,
    Trilobite,
    SeaScorpion,
    Nautilus,
    GiantWorm,
    // Prehistoric
    Raptor,
    Stegosaurus,
    Parasaur,
    Triceratops,
    Pterodactyl,
    CompyHealer,
    TRex,
    // Stone Age
    Caveman,
    Sabertooth,
    Mammoth,
    Shaman,
    GiantEagle,
    Dodo,
    WoollyRhino,
    // Ancient
    Legionary,
    Lion,
    WarElephant,
    EagleScout,
    Medicus,
    Minotaur,
    // Medieval
    Footman,
    Cavalry,
    Knight,
    Wyvern,
    Priest,
    Dragon,
}

pub struct EnemyStats {
    pub hp: f32,
    pub speed: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub gold_reward: u32,
    pub model_path: &'static str,
    pub model_scale: f32,
    pub is_flying: bool,
    /// Optional tint override — None means keep the model's original color.
    pub tint: Option<[f32; 3]>,
    /// Animation indices: [walk, idle, attack, death] — used for embedded GLTF anims (blobs).
    pub anim_indices: [usize; 4],
    /// External animation files [walk, idle, attack, death] — used for skinned enemies (non-blobs).
    /// When Some, `setup_enemy_animations` loads these instead of embedded anim_indices.
    pub anim_files: Option<[&'static str; 4]>,
    /// Extra Y-axis rotation for the model (radians). 0.0 = no rotation.
    pub rotation_y: f32,
    /// Whether this enemy type is a healer (gets HealerAura component).
    pub is_healer: bool,
    /// Optional bone remapping table: (mixamo_name, model_name) pairs.
    /// Used to retarget Mixamo animations onto non-Mixamo rigs.
    pub bone_map: Option<&'static [(&'static str, &'static str)]>,
}

/// Shared Mixamo animation files for skinned enemies.
const SKINNED_ANIMS: [&str; 4] = [
    "models/enemies/anims/crouch-walk.glb",
    "models/enemies/anims/idle.glb",
    "models/enemies/anims/attack.glb",
    "models/enemies/anims/die.glb",
];

/// Legionary animation — walk from embedded model clip, sword-parry for attack.
/// Uses the model's own GLB for walk/idle/death and sword-parry for attack.
const LEGIONARY_ANIMS: [&str; 4] = [
    "models/enemies/legionary.glb",        // walk (Animation0 = rig|rig|walk)
    "models/enemies/legionary.glb",        // idle (same walk clip)
    "models/enemies/anims/sword-parry.glb", // attack
    "models/enemies/legionary.glb",        // death (walk clip + rocking overlay)
];

pub fn enemy_stats(enemy_type: EnemyType) -> EnemyStats {
    match enemy_type {
        // =====================================================================
        // Primordial Era — blob enemies with embedded animations
        // =====================================================================
        EnemyType::Amoeba => EnemyStats {
            hp: 50.0, speed: 2.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 14, model_path: "models/enemies/PinkBlob.gltf",
            model_scale: 0.4, is_flying: false, tint: None,
            anim_indices: [7, 4, 0, 2],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Jellyfish => EnemyStats {
            hp: 40.0, speed: 2.5, armor: 0.0, magic_resist: 0.3,
            gold_reward: 19, model_path: "models/enemies/Hywirl.gltf",
            model_scale: 0.4, is_flying: true, tint: None,
            anim_indices: [1, 2, 3, 0],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Sporebloom => EnemyStats {
            hp: 30.0, speed: 1.5, armor: 0.0, magic_resist: 0.2,
            gold_reward: 19, model_path: "models/enemies/GreenBlob.gltf",
            model_scale: 0.4, is_flying: false, tint: Some([0.6, 0.3, 0.7]),
            anim_indices: [7, 4, 0, 2],
            anim_files: None, rotation_y: 0.0, is_healer: true, bone_map: None,
        },
        EnemyType::Trilobite => EnemyStats {
            hp: 80.0, speed: 2.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 24, model_path: "models/enemies/GreenBlob.gltf",
            model_scale: 0.6, is_flying: false, tint: None,
            anim_indices: [7, 4, 0, 2],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::SeaScorpion => EnemyStats {
            hp: 40.0, speed: 3.5, armor: 0.0, magic_resist: 0.0,
            gold_reward: 19, model_path: "models/enemies/GreenSpikyBlob.gltf",
            model_scale: 0.3, is_flying: false, tint: None,
            anim_indices: [7, 4, 0, 2],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Nautilus => EnemyStats {
            hp: 200.0, speed: 1.5, armor: 40.0, magic_resist: 0.0,
            gold_reward: 24, model_path: "models/enemies/GreenSpikyBlob.gltf",
            model_scale: 0.4, is_flying: false, tint: Some([0.2, 0.5, 1.0]),
            anim_indices: [7, 4, 0, 2],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::GiantWorm => EnemyStats {
            hp: 600.0, speed: 1.2, armor: 10.0, magic_resist: 0.1,
            gold_reward: 100, model_path: "models/enemies/GreenSpikyBlob.gltf",
            model_scale: 0.7, is_flying: false, tint: Some([0.7, 0.3, 0.2]),
            anim_indices: [7, 4, 0, 2],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },

        // =====================================================================
        // Prehistoric Era — skinned enemies with external Mixamo animations
        // =====================================================================
        EnemyType::Raptor => EnemyStats {
            hp: 35.0, speed: 3.2, armor: 0.0, magic_resist: 0.0,
            gold_reward: 16, model_path: "models/enemies/velociraptor.glb",
            model_scale: 0.15, is_flying: false, tint: Some([0.24, 0.53, 0.16]),
            anim_indices: [4, 2, 0, 1], // Run, Idle, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Stegosaurus => EnemyStats {
            hp: 60.0, speed: 2.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 16, model_path: "models/enemies/stegosaurus.glb",
            model_scale: 0.18, is_flying: false, tint: Some([0.24, 0.53, 0.42]),
            anim_indices: [4, 2, 0, 1], // Run, Idle, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Parasaur => EnemyStats {
            hp: 55.0, speed: 2.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 15, model_path: "models/enemies/parasaurolophus.glb",
            model_scale: 0.15, is_flying: false, tint: Some([0.33, 0.67, 0.27]),
            anim_indices: [4, 2, 0, 1], // Run, Idle, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Triceratops => EnemyStats {
            hp: 180.0, speed: 1.5, armor: 35.0, magic_resist: 0.1,
            gold_reward: 24, model_path: "models/enemies/triceratops.glb",
            model_scale: 0.2, is_flying: false, tint: Some([0.77, 0.47, 0.19]),
            anim_indices: [4, 2, 0, 1], // Run, Idle, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Pterodactyl => EnemyStats {
            hp: 40.0, speed: 2.5, armor: 0.0, magic_resist: 0.3,
            gold_reward: 19, model_path: "models/enemies/dragon.glb",
            model_scale: 0.35, is_flying: true, tint: Some([0.80, 0.27, 0.20]),
            anim_indices: [3, 3, 0, 2], // Flying, Flying, Attack, Death (dragon.glb)
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::CompyHealer => EnemyStats {
            hp: 30.0, speed: 2.0, armor: 0.0, magic_resist: 0.2,
            gold_reward: 19, model_path: "models/enemies/parasaurolophus.glb",
            model_scale: 0.12, is_flying: false, tint: Some([0.67, 0.33, 0.80]),
            anim_indices: [4, 2, 0, 1], // Run, Idle, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: true, bone_map: None,
        },
        EnemyType::TRex => EnemyStats {
            hp: 550.0, speed: 1.1, armor: 15.0, magic_resist: 0.1,
            gold_reward: 90, model_path: "models/enemies/trex.glb",
            model_scale: 0.35, is_flying: false, tint: Some([0.53, 0.27, 0.13]),
            anim_indices: [4, 2, 0, 1], // Run, Idle, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },

        // =====================================================================
        // Stone Age — skinned enemies
        // =====================================================================
        // Humanoid — Mixamo external anims
        EnemyType::Caveman => EnemyStats {
            hp: 55.0, speed: 2.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 16, model_path: "models/enemies/caveman.glb",
            model_scale: 1.0, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        // Animal — embedded Run anim + procedural rocking for attack/death
        EnemyType::Sabertooth => EnemyStats {
            hp: 40.0, speed: 3.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 17, model_path: "models/enemies/sabertooth.glb",
            model_scale: 1.2, is_flying: false, tint: None,
            anim_indices: [0, 0, 0, 0],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Mammoth => EnemyStats {
            hp: 200.0, speed: 1.5, armor: 40.0, magic_resist: 0.0,
            gold_reward: 26, model_path: "models/enemies/mammoth.glb",
            model_scale: 1.6, is_flying: false, tint: None,
            anim_indices: [2, 1, 0, 0], // Walk, Idle, HeadShake, HeadShake (no death anim)
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        // Humanoid — Mixamo external anims
        EnemyType::Shaman => EnemyStats {
            hp: 35.0, speed: 1.8, armor: 0.0, magic_resist: 0.2,
            gold_reward: 20, model_path: "models/enemies/shaman.glb",
            model_scale: 0.7, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: true, bone_map: None,
        },
        // Animal — procedural animation
        EnemyType::GiantEagle => EnemyStats {
            hp: 45.0, speed: 2.5, armor: 0.0, magic_resist: 0.3,
            gold_reward: 20, model_path: "models/enemies/eagle.glb",
            model_scale: 0.025, is_flying: true, tint: None,
            anim_indices: [0, 0, 0, 0], // single embedded fly anim for all states
            anim_files: None,
            rotation_y: 0.0, is_healer: false, bone_map: None, // facing handled by EnemyModelRotation on child
        },
        EnemyType::Dodo => EnemyStats {
            hp: 35.0, speed: 3.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 16, model_path: "models/enemies/dodo.glb",
            model_scale: 0.13, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::WoollyRhino => EnemyStats {
            hp: 650.0, speed: 1.0, armor: 20.0, magic_resist: 0.1,
            gold_reward: 100, model_path: "models/enemies/woolly-rhino.glb",
            model_scale: 2.5, is_flying: false, tint: None,
            anim_indices: [2, 2, 0, 1], // Walk, Walk, Bite, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },

        // =====================================================================
        // Ancient Era
        // =====================================================================
        // Humanoid — Mixamo external anims
        EnemyType::Legionary => EnemyStats {
            hp: 65.0, speed: 2.0, armor: 0.0, magic_resist: 0.1,
            gold_reward: 17, model_path: "models/enemies/legionary.glb",
            model_scale: 0.015, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: Some(LEGIONARY_ANIMS), rotation_y: 0.0, is_healer: false,
            bone_map: None,
        },
        // Animal — procedural animation
        EnemyType::Lion => EnemyStats {
            hp: 45.0, speed: 3.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 18, model_path: "models/enemies/lion.glb",
            model_scale: 1.5, is_flying: false, tint: Some([0.82, 0.62, 0.32]),
            anim_indices: [2, 3, 0, 1], // Run, Walk, Bite, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::WarElephant => EnemyStats {
            hp: 250.0, speed: 1.5, armor: 50.0, magic_resist: 0.15,
            gold_reward: 28, model_path: "models/enemies/war-elephant.glb",
            model_scale: 2.5, is_flying: false, tint: None,
            anim_indices: [2, 2, 0, 1], // Walk, Walk, Bite, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::EagleScout => EnemyStats {
            hp: 40.0, speed: 2.5, armor: 0.0, magic_resist: 0.3,
            gold_reward: 20, model_path: "models/enemies/eagle.glb",
            model_scale: 0.03, is_flying: true, tint: Some([0.45, 0.30, 0.18]),
            anim_indices: [0, 0, 0, 0], // single embedded fly anim for all states
            anim_files: None,
            rotation_y: 0.0, is_healer: false, bone_map: None, // facing handled by EnemyModelRotation on child
        },
        // Humanoid — Mixamo external anims
        EnemyType::Medicus => EnemyStats {
            hp: 40.0, speed: 1.8, armor: 0.0, magic_resist: 0.2,
            gold_reward: 22, model_path: "models/enemies/medicus.glb",
            model_scale: 0.7, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: true, bone_map: None,
        },
        EnemyType::Minotaur => EnemyStats {
            hp: 700.0, speed: 1.0, armor: 15.0, magic_resist: 0.15,
            gold_reward: 110, model_path: "models/enemies/minotaur-mixamo.glb",
            model_scale: 1.2, is_flying: false, tint: Some([0.45, 0.3, 0.2]),
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: false, bone_map: None,
        },

        // =====================================================================
        // Medieval Era
        // =====================================================================
        // Humanoid — Mixamo external anims
        EnemyType::Footman => EnemyStats {
            hp: 70.0, speed: 2.0, armor: 0.0, magic_resist: 0.1,
            gold_reward: 18, model_path: "models/enemies/footman.glb",
            model_scale: 1.0, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Cavalry => EnemyStats {
            hp: 50.0, speed: 3.0, armor: 0.0, magic_resist: 0.0,
            gold_reward: 19, model_path: "models/enemies/cavalry-horse.glb",
            model_scale: 0.5, is_flying: false, tint: None,
            anim_indices: [1, 3, 2, 3], // Trot, Rest, Gallop, Rest
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        EnemyType::Knight => EnemyStats {
            hp: 220.0, speed: 1.5, armor: 45.0, magic_resist: 0.2,
            gold_reward: 26, model_path: "models/enemies/knight.glb",
            model_scale: 0.8, is_flying: false, tint: None,
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        // Animal — procedural animation
        EnemyType::Wyvern => EnemyStats {
            hp: 50.0, speed: 2.5, armor: 0.0, magic_resist: 0.3,
            gold_reward: 22, model_path: "models/enemies/dragon.glb",
            model_scale: 0.3, is_flying: true, tint: None,
            anim_indices: [3, 3, 0, 2], // Flying, Flying, Attack, Death (same as Pterodactyl)
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
        // Humanoid — Mixamo external anims
        EnemyType::Priest => EnemyStats {
            hp: 35.0, speed: 1.8, armor: 0.0, magic_resist: 0.2,
            gold_reward: 22, model_path: "models/enemies/medicus.glb",
            model_scale: 0.6, is_flying: false, tint: Some([0.75, 0.6, 0.25]),
            anim_indices: [255; 4],
            anim_files: Some(SKINNED_ANIMS), rotation_y: 0.0, is_healer: true, bone_map: None,
        },
        EnemyType::Dragon => EnemyStats {
            hp: 800.0, speed: 1.2, armor: 10.0, magic_resist: 0.15,
            gold_reward: 120, model_path: "models/enemies/dragon.glb",
            model_scale: 0.7, is_flying: false, tint: Some([0.8, 0.2, 0.1]),
            anim_indices: [3, 3, 0, 2], // Flying, Flying, Attack, Death
            anim_files: None, rotation_y: 0.0, is_healer: false, bone_map: None,
        },
    }
}

// ---------------------------------------------------------------------------
// Tower stats
// ---------------------------------------------------------------------------

use crate::components::Element;

pub struct TowerStats {
    pub name: &'static str,
    pub cost: u32,
    pub damage: f32,
    pub attack_speed: f32,
    pub range: f32,
    pub model_path: &'static str,
    pub model_scale: f32,
}

pub fn tower_stats(element: Element, level: u8) -> TowerStats {
    match (element, level) {
        // Lightning — fast single-target magic
        (Element::Lightning, 0) => TowerStats {
            name: "Spark Tower", cost: 70, damage: 8.0,
            attack_speed: 1.2, range: 5.0,
            model_path: "models/towers/hive-turret.glb", model_scale: 0.75,
        },
        (Element::Lightning, 1) => TowerStats {
            name: "Bolt Tower", cost: 110, damage: 14.0,
            attack_speed: 1.3, range: 5.5,
            model_path: "models/towers/hive-turret.glb", model_scale: 0.85,
        },
        (Element::Lightning, 2) => TowerStats {
            name: "Storm Tower", cost: 160, damage: 22.0,
            attack_speed: 1.4, range: 6.0,
            model_path: "models/towers/hive-turret.glb", model_scale: 0.9,
        },

        // Earth — slow physical melee (uses projectiles until golems in Phase 4)
        (Element::Earth, 0) => TowerStats {
            name: "Clay Barracks", cost: 70, damage: 6.0,
            attack_speed: 0.8, range: 5.0,
            model_path: "models/towers/tower-earth.glb", model_scale: 1.2,
        },
        (Element::Earth, 1) => TowerStats {
            name: "Stone Barracks", cost: 110, damage: 9.0,
            attack_speed: 0.9, range: 5.5,
            model_path: "models/towers/tower-earth.glb", model_scale: 1.35,
        },
        (Element::Earth, 2) => TowerStats {
            name: "Golem Fortress", cost: 160, damage: 14.0,
            attack_speed: 1.0, range: 6.0,
            model_path: "models/towers/tower-earth.glb", model_scale: 1.5,
        },

        // Ice — slow magic with slow debuff
        (Element::Ice, 0) => TowerStats {
            name: "Frost Tower", cost: 100, damage: 15.0,
            attack_speed: 0.6, range: 5.0,
            model_path: "models/towers/tower-ice.glb", model_scale: 1.5,
        },
        (Element::Ice, 1) => TowerStats {
            name: "Ice Spire", cost: 160, damage: 27.0,
            attack_speed: 0.65, range: 5.5,
            model_path: "models/towers/tower-ice.glb", model_scale: 1.65,
        },
        (Element::Ice, 2) => TowerStats {
            name: "Blizzard Tower", cost: 240, damage: 45.0,
            attack_speed: 0.7, range: 6.0,
            model_path: "models/towers/tower-ice.glb", model_scale: 1.8,
        },

        // Fire — slow AoE splash with burn
        (Element::Fire, 0) => TowerStats {
            name: "Ember Cannon", cost: 125, damage: 14.0,
            attack_speed: 0.4, range: 6.0,
            model_path: "models/towers/tower-lightning.glb", model_scale: 2.25,
        },
        (Element::Fire, 1) => TowerStats {
            name: "Flame Mortar", cost: 190, damage: 22.0,
            attack_speed: 0.45, range: 6.5,
            model_path: "models/towers/tower-lightning.glb", model_scale: 2.4,
        },
        (Element::Fire, 2) => TowerStats {
            name: "Inferno Battery", cost: 280, damage: 42.0,
            attack_speed: 0.5, range: 7.0,
            model_path: "models/towers/tower-lightning.glb", model_scale: 2.55,
        },

        _ => unreachable!("Invalid tower level"),
    }
}

/// Base cost for a tower element (level 0).
pub fn tower_base_cost(element: Element) -> u32 {
    tower_stats(element, 0).cost
}

/// Projectile color per element.
pub fn element_color(element: Element) -> Color {
    match element {
        Element::Lightning => Color::srgb(1.0, 0.93, 0.27),  // yellow
        Element::Earth => Color::srgb(0.53, 0.67, 0.27),     // green-brown
        Element::Ice => Color::srgb(0.27, 0.8, 1.0),         // cyan
        Element::Fire => Color::srgb(1.0, 0.4, 0.13),        // orange
    }
}

/// Emissive glow per element (for projectiles).
pub fn element_emissive(element: Element) -> LinearRgba {
    match element {
        Element::Lightning => LinearRgba::new(1.0, 0.9, 0.3, 1.0),
        Element::Earth => LinearRgba::new(0.4, 0.5, 0.2, 1.0),
        Element::Ice => LinearRgba::new(0.3, 0.8, 1.0, 1.0),
        Element::Fire => LinearRgba::new(0.8, 0.3, 0.1, 1.0),
    }
}

/// Sell refund: 60% of total investment.
pub const SELL_REFUND_RATE: f32 = 0.6;

// ---------------------------------------------------------------------------
// Tower specializations (at max level)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TowerSpecialization {
    // Lightning
    StormSpire,     // Chain lightning jumps to 2 nearby enemies
    Railgun,        // Massive single-target damage, slower fire rate
    // Earth
    MountainKing,   // Fewer but tankier golems with AoE slam
    BrambleGrove,   // Slow+damage aura instead of golems
    // Ice
    BlizzardTower,  // Constant AoE slow field instead of projectiles
    ShatterMage,    // Slowed enemies take 3x crit damage
    // Fire
    InfernoCannon,  // Burns create napalm zones on ground
    MeteorTower,    // Huge single meteors, slower fire rate
}

pub struct SpecializationDef {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: u32,
}

/// Returns the two specialization options for a given element.
pub fn element_specializations(element: Element) -> [(TowerSpecialization, SpecializationDef); 2] {
    match element {
        Element::Lightning => [
            (TowerSpecialization::StormSpire, SpecializationDef {
                name: "Storm Spire",
                description: "Chain lightning jumps to 2 nearby enemies",
                cost: 200,
            }),
            (TowerSpecialization::Railgun, SpecializationDef {
                name: "Railgun Tower",
                description: "Massive damage, long range, slow fire",
                cost: 200,
            }),
        ],
        Element::Earth => [
            (TowerSpecialization::MountainKing, SpecializationDef {
                name: "Mountain King",
                description: "1 elite golem: 3x HP, 2x damage",
                cost: 200,
            }),
            (TowerSpecialization::BrambleGrove, SpecializationDef {
                name: "Bramble Grove",
                description: "Slow+damage aura, no golems",
                cost: 200,
            }),
        ],
        Element::Ice => [
            (TowerSpecialization::BlizzardTower, SpecializationDef {
                name: "Blizzard Tower",
                description: "Constant AoE slow field",
                cost: 250,
            }),
            (TowerSpecialization::ShatterMage, SpecializationDef {
                name: "Shatter Mage",
                description: "3x damage to slowed enemies",
                cost: 250,
            }),
        ],
        Element::Fire => [
            (TowerSpecialization::InfernoCannon, SpecializationDef {
                name: "Inferno Cannon",
                description: "Impacts leave burning ground",
                cost: 250,
            }),
            (TowerSpecialization::MeteorTower, SpecializationDef {
                name: "Meteor Tower",
                description: "Huge meteors, massive range",
                cost: 250,
            }),
        ],
    }
}

/// Spec upgrade info for upgrading from one spec level to the next.
pub struct SpecUpgrade {
    pub cost: u32,
    pub description: &'static str,
    /// Multiplicative damage bonus applied to tower's current damage.
    pub damage_mult: f32,
    /// Flat range bonus added.
    pub range_bonus: f32,
}

/// Maximum specialization level (1 = base spec, 2 and 3 are upgrades).
pub const MAX_SPEC_LEVEL: u8 = 3;

/// Returns the upgrade info for upgrading a spec to the given level (2 or 3).
/// Returns None if already at max or if `to_level` is invalid.
pub fn spec_upgrade_info(spec: TowerSpecialization, to_level: u8) -> Option<SpecUpgrade> {
    if to_level < 2 || to_level > MAX_SPEC_LEVEL { return None; }
    Some(match (spec, to_level) {
        // Lightning — Storm Spire
        (TowerSpecialization::StormSpire, 2) => SpecUpgrade {
            cost: 250, description: "Chains to 3 enemies", damage_mult: 1.25, range_bonus: 0.5,
        },
        (TowerSpecialization::StormSpire, 3) => SpecUpgrade {
            cost: 350, description: "Chains to 4 enemies, +stun", damage_mult: 1.3, range_bonus: 0.5,
        },
        // Lightning — Railgun
        (TowerSpecialization::Railgun, 2) => SpecUpgrade {
            cost: 250, description: "Pierces 2 enemies", damage_mult: 1.3, range_bonus: 1.0,
        },
        (TowerSpecialization::Railgun, 3) => SpecUpgrade {
            cost: 350, description: "Pierces all, armor shred", damage_mult: 1.35, range_bonus: 1.0,
        },
        // Earth — Mountain King
        (TowerSpecialization::MountainKing, 2) => SpecUpgrade {
            cost: 250, description: "Golem gains AoE slam", damage_mult: 1.25, range_bonus: 0.0,
        },
        (TowerSpecialization::MountainKing, 3) => SpecUpgrade {
            cost: 350, description: "Golem stuns on slam", damage_mult: 1.3, range_bonus: 0.0,
        },
        // Earth — Bramble Grove
        (TowerSpecialization::BrambleGrove, 2) => SpecUpgrade {
            cost: 250, description: "Wider aura, stronger slow", damage_mult: 1.25, range_bonus: 1.0,
        },
        (TowerSpecialization::BrambleGrove, 3) => SpecUpgrade {
            cost: 350, description: "Roots enemies briefly", damage_mult: 1.3, range_bonus: 1.0,
        },
        // Ice — Blizzard Tower
        (TowerSpecialization::BlizzardTower, 2) => SpecUpgrade {
            cost: 300, description: "Deeper freeze (30% slow)", damage_mult: 1.2, range_bonus: 0.5,
        },
        (TowerSpecialization::BlizzardTower, 3) => SpecUpgrade {
            cost: 400, description: "Frostbite: frozen deal 2x dmg", damage_mult: 1.25, range_bonus: 0.5,
        },
        // Ice — Shatter Mage
        (TowerSpecialization::ShatterMage, 2) => SpecUpgrade {
            cost: 300, description: "4x crit on slowed", damage_mult: 1.25, range_bonus: 0.5,
        },
        (TowerSpecialization::ShatterMage, 3) => SpecUpgrade {
            cost: 400, description: "5x crit, AoE shatter on kill", damage_mult: 1.3, range_bonus: 0.5,
        },
        // Fire — Inferno Cannon
        (TowerSpecialization::InfernoCannon, 2) => SpecUpgrade {
            cost: 300, description: "Larger burn zones", damage_mult: 1.25, range_bonus: 0.5,
        },
        (TowerSpecialization::InfernoCannon, 3) => SpecUpgrade {
            cost: 400, description: "Burn zones stack damage", damage_mult: 1.3, range_bonus: 0.5,
        },
        // Fire — Meteor Tower
        (TowerSpecialization::MeteorTower, 2) => SpecUpgrade {
            cost: 300, description: "Wider blast radius", damage_mult: 1.3, range_bonus: 1.0,
        },
        (TowerSpecialization::MeteorTower, 3) => SpecUpgrade {
            cost: 400, description: "Volcanic eruption on impact", damage_mult: 1.35, range_bonus: 1.0,
        },
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Wave definitions for Level 1
// ---------------------------------------------------------------------------

pub struct WaveDefinition {
    pub groups: Vec<WaveGroup>,
    pub early_call_bonus: u32,
}

pub struct WaveGroup {
    pub enemy_type: EnemyType,
    pub count: u32,
    pub interval: f32,
    pub delay: f32, // seconds before this group starts spawning (relative to pulse start)
    pub pulse: u32, // which pulse this group belongs to (0-indexed)
}

/// All 10 waves for Level 1 — pulse-based for fight-breathe-fight rhythm.
pub fn level1_waves() -> Vec<WaveDefinition> {
    vec![
        // Wave 1 — single pulse intro
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 5, interval: 1.0, delay: 0.0, pulse: 0 },
        ]},
        // Wave 2 — single pulse
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
        ]},
        // Wave 3 — 2 pulses, introduce fast enemies
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 6, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 3, interval: 0.6, delay: 0.0, pulse: 1 },
        ]},
        // Wave 4 — 2 pulses, introduce trilobites
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 6, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 5, interval: 0.7, delay: 0.0, pulse: 1 },
        ]},
        // Wave 5 — 2 pulses, introduce armored
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 8, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 4, interval: 0.5, delay: 2.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 2, interval: 1.5, delay: 0.0, pulse: 1 },
        ]},
        // Wave 6 — 2 pulses, introduce healers
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 4, interval: 1.2, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 5, interval: 0.5, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 5, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 2, interval: 1.2, delay: 1.0, pulse: 1 },
        ]},
        // Wave 7 — 2 pulses, introduce flying
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 8, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 3, interval: 0.7, delay: 2.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 3, interval: 0.7, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 3, interval: 1.0, delay: 1.5, pulse: 1 },
        ]},
        // Wave 8 — 3 pulses
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 5, interval: 1.0, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 6, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 3, interval: 1.0, delay: 1.5, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 8, interval: 0.6, delay: 0.0, pulse: 2 },
        ]},
        // Wave 9 — 3 pulses, heavy mix
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 10, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 4, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 2, interval: 1.0, delay: 1.5, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 5, interval: 0.8, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 6, interval: 0.5, delay: 1.0, pulse: 2 },
        ]},
        // Wave 10 — 3 pulses, final assault
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 6, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 5, interval: 0.4, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 4, interval: 0.8, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 5, interval: 0.4, delay: 1.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 4, interval: 0.7, delay: 0.0, pulse: 2 },
        ]},
    ]
}

// ---------------------------------------------------------------------------
// Theme colors
// ---------------------------------------------------------------------------

pub struct ThemeColors {
    pub ground: Color,
    pub path: Color,
    pub sky: Color,
}

pub fn primordial_theme() -> ThemeColors {
    ThemeColors {
        ground: Color::srgb(0.071, 0.031, 0.094),
        path: Color::srgb(0.165, 0.102, 0.188),
        sky: Color::srgb(0.102, 0.055, 0.141),
    }
}

pub fn prehistoric_theme() -> ThemeColors {
    ThemeColors {
        ground: Color::srgb(0.184, 0.333, 0.133), // 0x2f5522
        path: Color::srgb(0.722, 0.604, 0.416),   // 0xb89a6a
        sky: Color::srgb(0.529, 0.808, 0.922),     // 0x87ceeb
    }
}

pub fn frozen_tundra_theme() -> ThemeColors {
    ThemeColors {
        ground: Color::srgb(0.816, 0.867, 0.910), // 0xd0dde8
        path: Color::srgb(0.541, 0.604, 0.667),   // 0x8a9aaa
        sky: Color::srgb(0.722, 0.816, 0.910),     // 0xb8d0e8
    }
}

pub fn ancient_theme() -> ThemeColors {
    ThemeColors {
        ground: Color::srgb(0.769, 0.659, 0.408), // 0xc4a868
        path: Color::srgb(0.604, 0.565, 0.502),   // 0x9a9080
        sky: Color::srgb(0.420, 0.686, 0.878),     // 0x6bafe0
    }
}

pub fn medieval_theme() -> ThemeColors {
    ThemeColors {
        ground: Color::srgb(0.227, 0.408, 0.157), // 0x3a6828
        path: Color::srgb(0.541, 0.439, 0.314),   // 0x8a7050
        sky: Color::srgb(0.478, 0.604, 0.690),     // 0x7a9ab0
    }
}

/// Default HP scale per wave — overridden by LevelStartConfig per level.
pub const WAVE_HP_SCALE: f32 = 0.10;
/// Default Speed scale per wave — overridden by LevelStartConfig per level.
pub const WAVE_SPEED_SCALE: f32 = 0.015;

// ---------------------------------------------------------------------------
// Hero definitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeroType {
    SacredMaiden,
    IceHulk,
    NorthernOutsider,
    Pharaoh,
    ScarletMagus,
}

pub struct HeroStats {
    pub name: &'static str,
    pub hp: f32,
    pub damage: f32,
    pub attack_speed: f32,  // attacks per second
    pub attack_range: f32,
    pub move_speed: f32,
    pub model_path: &'static str,
    pub model_scale: f32,
    /// Extra rotation applied to the model root (in radians, around X axis).
    /// Some Mixamo models export with wrong up-axis orientation.
    pub model_rotation_x: f32,
    /// Y offset for the model (positive = up). Fixes models sunk into or floating above ground.
    pub model_y_offset: f32,
    pub respawn_time: f32,
    pub idle_anim: &'static str,
    pub attack_anim: &'static str,
    pub run_anim: &'static str,
    /// When true, reset all bone translations/scales to bind pose every frame,
    /// keeping only rotation from the animation. Needed for models whose bone
    /// translations differ by ~100x from the animation files (Mixamo scale mismatch).
    pub rotation_only_anims: bool,
    /// When true, skip Hips root-motion stripping and per-frame Hips reset.
    /// Needed when the animation clips have compatible Hips values for this model
    /// but the model's bind-pose Hips are in a different scale than standard Mixamo.
    pub skip_root_motion_cancel: bool,
}

pub fn hero_stats(hero_type: HeroType) -> HeroStats {
    match hero_type {
        HeroType::SacredMaiden => HeroStats {
            name: "Aethon",
            hp: 300.0, damage: 12.0, attack_speed: 1.2,
            attack_range: 2.5, move_speed: 5.0,
            model_path: "models/heroes/sacred-maiden.glb",
            model_scale: 1.0, model_rotation_x: 0.0, model_y_offset: 0.0,
            respawn_time: 15.0,
            idle_anim: "models/heroes/anims/maiden-idle.glb",
            attack_anim: "models/heroes/anims/maiden-melee-kick.glb",
            run_anim: "models/enemies/anims/run.glb",
            rotation_only_anims: true,
            skip_root_motion_cancel: true,
        },
        HeroType::IceHulk => HeroStats {
            name: "Cryo",
            hp: 600.0, damage: 20.0, attack_speed: 0.7,
            attack_range: 2.0, move_speed: 3.5,
            model_path: "models/heroes/ice-hulk.glb",
            model_scale: 1.2, model_rotation_x: 0.0, model_y_offset: 1.2,
            respawn_time: 20.0,
            idle_anim: "models/heroes/anims/mutant-idle.glb",
            attack_anim: "models/heroes/anims/melee-combo.glb",
            run_anim: "models/enemies/anims/run.glb",
            rotation_only_anims: true,
            skip_root_motion_cancel: true,
        },
        HeroType::NorthernOutsider => HeroStats {
            name: "Gorath",
            hp: 250.0, damage: 18.0, attack_speed: 0.7,
            attack_range: 2.5, move_speed: 6.0,
            model_path: "models/heroes/northern-outsider.glb",
            model_scale: 0.009, model_rotation_x: 0.0, model_y_offset: 0.0,
            respawn_time: 12.0,
            idle_anim: "models/heroes/anims/mutant-idle.glb",
            attack_anim: "models/heroes/anims/melee-combo.glb",
            run_anim: "models/enemies/anims/run.glb",
            rotation_only_anims: true,
            skip_root_motion_cancel: true,
        },
        HeroType::Pharaoh => HeroStats {
            name: "Voltra",
            hp: 350.0, damage: 15.0, attack_speed: 1.0,
            attack_range: 3.0, move_speed: 4.5,
            model_path: "models/heroes/pharaoh.glb",
            model_scale: 0.015, model_rotation_x: std::f32::consts::FRAC_PI_2, model_y_offset: 1.5,
            respawn_time: 18.0,
            idle_anim: "models/heroes/anims/maiden-idle.glb",
            attack_anim: "models/heroes/anims/maiden-melee-kick.glb",
            run_anim: "models/enemies/anims/run.glb",
            rotation_only_anims: true,
            skip_root_motion_cancel: true,
        },
        HeroType::ScarletMagus => HeroStats {
            name: "Ignis",
            hp: 200.0, damage: 25.0, attack_speed: 0.8,
            attack_range: 5.0, move_speed: 4.0,
            model_path: "models/heroes/scarlet-magus.glb",
            model_scale: 1.0, model_rotation_x: 0.0, model_y_offset: 0.0,
            respawn_time: 14.0,
            idle_anim: "models/heroes/anims/maiden-idle.glb",
            attack_anim: "models/heroes/anims/maiden-melee-kick.glb",
            run_anim: "models/enemies/anims/run.glb",
            rotation_only_anims: true,
            skip_root_motion_cancel: true,
        },
    }
}

/// Default hero spawn position (near the path entrance).
pub fn hero_spawn_pos() -> Vec3 {
    Vec3::new(-16.0, 0.0, 3.0)
}

// ---------------------------------------------------------------------------
// Level 2 — Primordial Depths
// ---------------------------------------------------------------------------

pub fn level2_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 8.0),
        Vec3::new(-10.0, 0.0, 8.0),
        Vec3::new(-7.0, 0.0, 2.0),
        Vec3::new(-3.0, 0.0, -4.0),
        Vec3::new(3.0, 0.0, -6.0),
        Vec3::new(7.0, 0.0, -2.0),
        Vec3::new(5.0, 0.0, 5.0),
        Vec3::new(10.0, 0.0, 7.0),
        Vec3::new(14.0, 0.0, 2.0),
        Vec3::new(18.0, 0.0, -4.0),
    ]
}

pub fn level2_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-12.0, 0.0, 4.0),
        Vec3::new(-8.0, 0.0, -2.0),
        Vec3::new(-5.0, 0.0, 6.0),
        Vec3::new(-1.0, 0.0, -8.0),
        Vec3::new(5.0, 0.0, -8.0),
        Vec3::new(9.0, 0.0, -4.0),
        Vec3::new(3.0, 0.0, 7.0),
        Vec3::new(8.0, 0.0, 9.0),
        Vec3::new(14.0, 0.0, 6.5),
        Vec3::new(18.5, 0.0, -0.5),
    ]
}

/// Level 2 hero spawn position.
pub fn level2_hero_spawn() -> Vec3 {
    Vec3::new(-16.0, 0.0, 10.0)
}

/// 10 waves for Level 2 — harder mix, GiantWorm boss finale.
pub fn level2_waves() -> Vec<WaveDefinition> {
    vec![
        // Wave 1 — warm up
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
        ]},
        // Wave 2 — fast rush
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 6, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Amoeba, count: 4, interval: 0.7, delay: 0.0, pulse: 1 },
        ]},
        // Wave 3 — armored push
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 8, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 3, interval: 1.2, delay: 0.0, pulse: 1 },
        ]},
        // Wave 4 — flying swarm
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 6, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 5, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        // Wave 5 — healers + tanks
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 5, interval: 1.0, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 3, interval: 1.0, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 6, interval: 0.6, delay: 0.0, pulse: 1 },
        ]},
        // Wave 6 — mixed assault
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 10, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 4, interval: 0.8, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 4, interval: 1.0, delay: 2.0, pulse: 1 },
        ]},
        // Wave 7 — heavy armor + healer support
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 4, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 8, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        // Wave 8 — relentless
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 12, interval: 0.3, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 6, interval: 0.6, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 6, interval: 0.8, delay: 0.0, pulse: 2 },
        ]},
        // Wave 9 — everything
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 6, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 3, interval: 1.0, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 8, interval: 0.4, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 5, interval: 0.7, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: EnemyType::Trilobite, count: 10, interval: 0.5, delay: 1.0, pulse: 2 },
        ]},
        // Wave 10 — GiantWorm boss + escort
        WaveDefinition { early_call_bonus: 40, groups: vec![
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 4, interval: 1.0, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sporebloom, count: 2, interval: 1.0, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::GiantWorm, count: 1, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::SeaScorpion, count: 8, interval: 0.4, delay: 1.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Jellyfish, count: 5, interval: 0.6, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: EnemyType::Nautilus, count: 4, interval: 0.8, delay: 1.5, pulse: 2 },
        ]},
    ]
}

// ---------------------------------------------------------------------------
// Level 3 — Jurassic Jungle (Prehistoric)
// ---------------------------------------------------------------------------

pub fn level3_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, -5.0),
        Vec3::new(-12.0, 0.0, -5.0),
        Vec3::new(-8.0, 0.0, 0.0),
        Vec3::new(-4.0, 0.0, 6.0),
        Vec3::new(2.0, 0.0, 6.0),
        Vec3::new(6.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, -4.0),
        Vec3::new(14.0, 0.0, -4.0),
        Vec3::new(18.0, 0.0, 0.0),
    ]
}

pub fn level3_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-14.0, 0.0, -8.0),
        Vec3::new(-10.0, 0.0, 3.0),
        Vec3::new(-6.0, 0.0, -3.0),
        Vec3::new(-2.0, 0.0, 9.0),
        Vec3::new(1.5, 0.0, 1.5),
        Vec3::new(8.0, 0.0, -7.0),
        Vec3::new(8.0, 0.0, 3.0),
        Vec3::new(12.0, 0.0, -7.0),
        Vec3::new(16.0, 0.0, 3.0),
    ]
}

pub fn level3_waves() -> Vec<WaveDefinition> {
    vec![
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Stegosaurus, count: 6, interval: 1.0, delay: 0.0, pulse: 0 },
        ]},
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Stegosaurus, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 3, interval: 0.6, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Parasaur, count: 8, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 5, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Stegosaurus, count: 6, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 2, interval: 1.2, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Raptor, count: 8, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 3, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 3, interval: 0.8, delay: 1.5, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Parasaur, count: 10, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 2, interval: 1.0, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 4, interval: 1.0, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Stegosaurus, count: 8, interval: 0.6, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 4, interval: 0.7, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 6, interval: 0.4, delay: 1.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 5, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 3, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 10, interval: 0.3, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Parasaur, count: 12, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 4, interval: 0.8, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 5, interval: 0.6, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 3, interval: 0.8, delay: 1.0, pulse: 2 },
        ]},
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 6, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 3, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 10, interval: 0.3, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 5, interval: 0.5, delay: 0.0, pulse: 2 },
        ]},
    ]
}

// ---------------------------------------------------------------------------
// Level 4 — Volcanic Pass (Prehistoric)
// ---------------------------------------------------------------------------

pub fn level4_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 4.0),
        Vec3::new(-10.0, 0.0, 4.0),
        Vec3::new(-6.0, 0.0, -3.0),
        Vec3::new(0.0, 0.0, -6.0),
        Vec3::new(6.0, 0.0, -3.0),
        Vec3::new(10.0, 0.0, 4.0),
        Vec3::new(18.0, 0.0, 4.0),
    ]
}

pub fn level4_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-14.0, 0.0, 7.0),
        Vec3::new(-12.0, 0.0, 0.0),
        Vec3::new(-8.0, 0.0, -6.0),
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, -9.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(8.0, 0.0, -6.0),
        Vec3::new(8.0, 0.0, 7.0),
        Vec3::new(14.0, 0.0, 7.0),
    ]
}

pub fn level4_waves() -> Vec<WaveDefinition> {
    vec![
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Parasaur, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
        ]},
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Raptor, count: 6, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Stegosaurus, count: 6, interval: 0.7, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 4, interval: 1.0, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 6, interval: 0.4, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 5, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Parasaur, count: 8, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 5, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 3, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 8, interval: 0.3, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Stegosaurus, count: 10, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 5, interval: 0.6, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 4, interval: 1.0, delay: 1.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Raptor, count: 12, interval: 0.3, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 6, interval: 0.7, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 3, interval: 0.8, delay: 1.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Parasaur, count: 12, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 6, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 5, interval: 0.8, delay: 0.0, pulse: 2 },
        ]},
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 8, interval: 0.6, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 4, interval: 0.7, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 12, interval: 0.3, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 6, interval: 0.5, delay: 0.0, pulse: 2 },
        ]},
        // Wave 10 — T-Rex boss
        WaveDefinition { early_call_bonus: 40, groups: vec![
            WaveGroup { enemy_type: EnemyType::Triceratops, count: 4, interval: 0.8, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::CompyHealer, count: 2, interval: 1.0, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::TRex, count: 1, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Raptor, count: 10, interval: 0.3, delay: 1.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Pterodactyl, count: 5, interval: 0.5, delay: 0.0, pulse: 2 },
        ]},
    ]
}

// ---------------------------------------------------------------------------
// Level 5 — Frozen Tundra (Stone Age)
// ---------------------------------------------------------------------------

pub fn level5_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 0.0),
        Vec3::new(-12.0, 0.0, -6.0),
        Vec3::new(-6.0, 0.0, -6.0),
        Vec3::new(-2.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 6.0),
        Vec3::new(8.0, 0.0, 6.0),
        Vec3::new(12.0, 0.0, 0.0),
        Vec3::new(18.0, 0.0, 0.0),
    ]
}

pub fn level5_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-15.0, 0.0, 0.0),
        Vec3::new(-14.0, 0.0, -9.0),
        Vec3::new(-8.0, 0.0, -3.0),
        Vec3::new(-4.0, 0.0, -9.0),
        Vec3::new(-2.0, 0.0, 4.5),
        Vec3::new(5.0, 0.0, 9.0),
        Vec3::new(5.0, 0.0, 3.0),
        Vec3::new(11.5, 0.0, 4.5),
        Vec3::new(14.0, 0.0, -3.0),
    ]
}

pub fn level5_waves() -> Vec<WaveDefinition> {
    vec![
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
        ]},
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 6, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Dodo, count: 4, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 6, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Caveman, count: 8, interval: 0.6, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 3, interval: 1.2, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Dodo, count: 6, interval: 0.4, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 8, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 4, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 3, interval: 0.8, delay: 1.5, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 8, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 3, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 5, interval: 0.8, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 12, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 4, interval: 0.7, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 4, interval: 0.8, delay: 1.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 6, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 3, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 10, interval: 0.3, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 5, interval: 0.6, delay: 0.0, pulse: 2 },
        ]},
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 18, interval: 0.25, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 7, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 4, interval: 0.7, delay: 0.5, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 6, interval: 0.45, delay: 0.0, pulse: 2 },
        ]},
        WaveDefinition { early_call_bonus: 35, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 8, interval: 0.6, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 4, interval: 0.7, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 15, interval: 0.25, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 7, interval: 0.45, delay: 0.0, pulse: 2 },
        ]},
    ]
}

// ---------------------------------------------------------------------------
// Level 6 — Glacier Gorge (Stone Age)
// ---------------------------------------------------------------------------

pub fn level6_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 6.0),
        Vec3::new(-10.0, 0.0, 6.0),
        Vec3::new(-6.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -6.0),
        Vec3::new(6.0, 0.0, -6.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(14.0, 0.0, 6.0),
        Vec3::new(18.0, 0.0, 6.0),
    ]
}

pub fn level6_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-14.0, 0.0, 9.0),
        Vec3::new(-12.0, 0.0, 2.0),
        Vec3::new(-8.0, 0.0, -3.0),
        Vec3::new(-3.0, 0.0, 3.0),
        Vec3::new(-3.0, 0.0, -9.0),
        Vec3::new(3.0, 0.0, -3.0),
        Vec3::new(8.0, 0.0, -9.0),
        Vec3::new(8.0, 0.0, 3.0),
        Vec3::new(12.0, 0.0, 9.0),
        Vec3::new(16.0, 0.0, 3.0),
    ]
}

pub fn level6_waves() -> Vec<WaveDefinition> {
    vec![
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 10, interval: 0.7, delay: 0.0, pulse: 0 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 8, interval: 0.4, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Caveman, count: 6, interval: 0.6, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 4, interval: 1.0, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Dodo, count: 8, interval: 0.4, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 10, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 4, interval: 0.7, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 3, interval: 1.0, delay: 1.5, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 6, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 3, interval: 0.8, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 10, interval: 0.3, delay: 0.0, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Dodo, count: 12, interval: 0.3, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 5, interval: 0.8, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 5, interval: 0.6, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 3, interval: 0.8, delay: 1.0, pulse: 2 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Caveman, count: 15, interval: 0.3, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 6, interval: 0.6, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 4, interval: 0.7, delay: 0.5, pulse: 1 },
        ]},
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 10, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 5, interval: 0.6, delay: 1.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 7, interval: 0.45, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 15, interval: 0.25, delay: 0.0, pulse: 2 },
        ]},
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 18, interval: 0.18, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 8, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 7, interval: 0.45, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 5, interval: 0.6, delay: 0.5, pulse: 2 },
        ]},
        // Wave 10 — Woolly Rhino boss
        WaveDefinition { early_call_bonus: 40, groups: vec![
            WaveGroup { enemy_type: EnemyType::Mammoth, count: 5, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::Shaman, count: 3, interval: 0.8, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: EnemyType::WoollyRhino, count: 1, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::Sabertooth, count: 12, interval: 0.25, delay: 1.0, pulse: 1 },
            WaveGroup { enemy_type: EnemyType::GiantEagle, count: 6, interval: 0.45, delay: 0.0, pulse: 2 },
        ]},
    ]
}

// ---------------------------------------------------------------------------
// Levels 7-10 — use make_waves() helper
// ---------------------------------------------------------------------------

/// Roster for generating standardized waves via `make_waves()`.
pub struct WaveRoster {
    pub grunt: EnemyType,
    pub fast: EnemyType,
    pub tank: EnemyType,
    pub flyer: EnemyType,
    pub healer: EnemyType,
    pub boss: Option<EnemyType>,
}

/// Generates 10 standardized waves from a roster definition.
pub fn make_waves(r: &WaveRoster) -> Vec<WaveDefinition> {
    let wave10 = if let Some(boss) = r.boss {
        WaveDefinition { early_call_bonus: 40, groups: vec![
            WaveGroup { enemy_type: r.tank, count: 5, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.healer, count: 3, interval: 0.8, delay: 1.5, pulse: 0 },
            WaveGroup { enemy_type: boss, count: 1, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.fast, count: 12, interval: 0.25, delay: 1.0, pulse: 1 },
            WaveGroup { enemy_type: r.flyer, count: 6, interval: 0.5, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: r.tank, count: 5, interval: 0.6, delay: 1.0, pulse: 2 },
        ]}
    } else {
        WaveDefinition { early_call_bonus: 40, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 22, interval: 0.18, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.tank, count: 10, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.flyer, count: 7, interval: 0.45, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: r.healer, count: 5, interval: 0.6, delay: 1.0, pulse: 2 },
        ]}
    };

    vec![
        // Wave 1: 8 grunts
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 8, interval: 0.8, delay: 0.0, pulse: 0 },
        ]},
        // Wave 2: 6 grunts + 3 fast
        WaveDefinition { early_call_bonus: 10, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 6, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 3, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        // Wave 3: 8 grunts + 4 fast
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 8, interval: 0.6, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 4, interval: 0.5, delay: 0.0, pulse: 1 },
        ]},
        // Wave 4: 5 grunts + 3 fast + 2 tank
        WaveDefinition { early_call_bonus: 15, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 5, interval: 0.7, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 3, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.tank, count: 2, interval: 1.2, delay: 0.0, pulse: 1 },
        ]},
        // Wave 5: 8 grunts + 4 fast + 3 tank + 2 flyer
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 8, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 4, interval: 0.4, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.tank, count: 3, interval: 1.0, delay: 1.0, pulse: 1 },
            WaveGroup { enemy_type: r.flyer, count: 2, interval: 0.8, delay: 0.0, pulse: 2 },
        ]},
        // Wave 6: 10 grunts + 5 fast + 3 tank + 2 healer
        WaveDefinition { early_call_bonus: 20, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 10, interval: 0.5, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 5, interval: 0.4, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.tank, count: 3, interval: 1.0, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.healer, count: 2, interval: 1.0, delay: 1.5, pulse: 1 },
        ]},
        // Wave 7: 14 grunts + 7 fast + 5 tank + 4 flyer + 3 healer
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 14, interval: 0.35, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 7, interval: 0.3, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.tank, count: 5, interval: 0.7, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.flyer, count: 4, interval: 0.6, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: r.healer, count: 3, interval: 0.8, delay: 1.0, pulse: 2 },
        ]},
        // Wave 8: 15 grunts + 10 fast + 6 tank + 5 flyer + 3 healer
        WaveDefinition { early_call_bonus: 25, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 15, interval: 0.35, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 10, interval: 0.25, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.tank, count: 6, interval: 0.6, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.flyer, count: 5, interval: 0.5, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: r.healer, count: 3, interval: 0.8, delay: 1.0, pulse: 2 },
        ]},
        // Wave 9: 18 grunts + 10 fast + 7 tank + 6 flyer + 4 healer
        WaveDefinition { early_call_bonus: 30, groups: vec![
            WaveGroup { enemy_type: r.grunt, count: 18, interval: 0.25, delay: 0.0, pulse: 0 },
            WaveGroup { enemy_type: r.fast, count: 10, interval: 0.25, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.tank, count: 7, interval: 0.5, delay: 0.0, pulse: 1 },
            WaveGroup { enemy_type: r.flyer, count: 6, interval: 0.45, delay: 0.0, pulse: 2 },
            WaveGroup { enemy_type: r.healer, count: 4, interval: 0.7, delay: 1.0, pulse: 2 },
        ]},
        // Wave 10: boss + escort (or heavy wave if no boss)
        wave10,
    ]
}

// ---------------------------------------------------------------------------
// Level 7 — Via Romana (Ancient)
// ---------------------------------------------------------------------------

pub fn level7_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, -3.0),
        Vec3::new(-10.0, 0.0, -3.0),
        Vec3::new(-6.0, 0.0, 4.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(4.0, 0.0, -2.0),
        Vec3::new(10.0, 0.0, -2.0),
        Vec3::new(14.0, 0.0, 5.0),
        Vec3::new(18.0, 0.0, 5.0),
    ]
}

pub fn level7_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-14.0, 0.0, -6.0),
        Vec3::new(-12.0, 0.0, 1.0),
        Vec3::new(-8.0, 0.0, 7.0),
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 7.0),
        Vec3::new(2.0, 0.0, -5.0),
        Vec3::new(7.0, 0.0, 2.0),
        Vec3::new(12.0, 0.0, -5.0),
        Vec3::new(12.0, 0.0, 8.0),
        Vec3::new(16.0, 0.0, 2.0),
    ]
}

pub fn level7_waves() -> Vec<WaveDefinition> {
    make_waves(&WaveRoster {
        grunt: EnemyType::Legionary,
        fast: EnemyType::Lion,
        tank: EnemyType::WarElephant,
        flyer: EnemyType::EagleScout,
        healer: EnemyType::Medicus,
        boss: None,
    })
}

// ---------------------------------------------------------------------------
// Level 8 — Colosseum (Ancient)
// ---------------------------------------------------------------------------

pub fn level8_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 5.0),
        Vec3::new(-12.0, 0.0, 5.0),
        Vec3::new(-8.0, 0.0, -2.0),
        Vec3::new(-2.0, 0.0, -6.0),
        Vec3::new(4.0, 0.0, -2.0),
        Vec3::new(8.0, 0.0, 5.0),
        Vec3::new(14.0, 0.0, 5.0),
        Vec3::new(18.0, 0.0, 0.0),
    ]
}

pub fn level8_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-14.0, 0.0, 8.0),
        Vec3::new(-14.0, 0.0, 1.0),
        Vec3::new(-10.0, 0.0, -5.0),
        Vec3::new(-5.0, 0.0, 2.0),
        Vec3::new(-5.0, 0.0, -9.0),
        Vec3::new(1.0, 0.0, -0.5),
        Vec3::new(6.0, 0.0, -5.0),
        Vec3::new(6.0, 0.0, 8.0),
        Vec3::new(11.0, 0.0, 2.0),
        Vec3::new(16.0, 0.0, -3.0),
    ]
}

pub fn level8_waves() -> Vec<WaveDefinition> {
    make_waves(&WaveRoster {
        grunt: EnemyType::Legionary,
        fast: EnemyType::Lion,
        tank: EnemyType::WarElephant,
        flyer: EnemyType::EagleScout,
        healer: EnemyType::Medicus,
        boss: Some(EnemyType::Minotaur),
    })
}

// ---------------------------------------------------------------------------
// Level 9 — Castle Approach (Medieval)
// ---------------------------------------------------------------------------

pub fn level9_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, 0.0),
        Vec3::new(-12.0, 0.0, 6.0),
        Vec3::new(-6.0, 0.0, 6.0),
        Vec3::new(-2.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, -6.0),
        Vec3::new(8.0, 0.0, -6.0),
        Vec3::new(12.0, 0.0, 0.0),
        Vec3::new(18.0, 0.0, 0.0),
    ]
}

pub fn level9_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-15.0, 0.0, 0.0),
        Vec3::new(-14.0, 0.0, 9.0),
        Vec3::new(-8.0, 0.0, 3.0),
        Vec3::new(-4.0, 0.0, 9.0),
        Vec3::new(1.5, 0.0, 0.0),
        Vec3::new(5.0, 0.0, -9.0),
        Vec3::new(5.0, 0.0, -2.0),
        Vec3::new(11.5, 0.0, -7.5),
        Vec3::new(10.0, 0.0, 3.0),
        Vec3::new(15.0, 0.0, -3.0),
    ]
}

pub fn level9_waves() -> Vec<WaveDefinition> {
    make_waves(&WaveRoster {
        grunt: EnemyType::Footman,
        fast: EnemyType::Cavalry,
        tank: EnemyType::Knight,
        flyer: EnemyType::Wyvern,
        healer: EnemyType::Priest,
        boss: None,
    })
}

// ---------------------------------------------------------------------------
// Level 10 — Dragon's Lair (Medieval)
// ---------------------------------------------------------------------------

pub fn level10_path() -> Vec<Vec3> {
    vec![
        Vec3::new(-18.0, 0.0, -4.0),
        Vec3::new(-12.0, 0.0, -4.0),
        Vec3::new(-8.0, 0.0, 3.0),
        Vec3::new(-2.0, 0.0, 7.0),
        Vec3::new(4.0, 0.0, 3.0),
        Vec3::new(8.0, 0.0, -4.0),
        Vec3::new(14.0, 0.0, -4.0),
        Vec3::new(18.0, 0.0, 2.0),
    ]
}

pub fn level10_build_spots() -> Vec<Vec3> {
    vec![
        Vec3::new(-14.0, 0.0, -7.0),
        Vec3::new(-14.0, 0.0, 0.0),
        Vec3::new(-10.0, 0.0, 6.0),
        Vec3::new(-5.0, 0.0, 0.0),
        Vec3::new(-5.0, 0.0, 9.0),
        Vec3::new(0.0, 0.0, 1.5),
        Vec3::new(6.0, 0.0, 7.0),
        Vec3::new(6.0, 0.0, -7.0),
        Vec3::new(11.0, 0.0, 0.0),
        Vec3::new(16.0, 0.0, -7.0),
        Vec3::new(16.0, 0.0, 5.0),
    ]
}

pub fn level10_waves() -> Vec<WaveDefinition> {
    make_waves(&WaveRoster {
        grunt: EnemyType::Footman,
        fast: EnemyType::Cavalry,
        tank: EnemyType::Knight,
        flyer: EnemyType::Wyvern,
        healer: EnemyType::Priest,
        boss: Some(EnemyType::Dragon),
    })
}

// ---------------------------------------------------------------------------
// Level dispatcher functions
// ---------------------------------------------------------------------------

pub fn level_path(level: u32) -> Vec<Vec3> {
    match level {
        1 => level1_path(),
        2 => level2_path(),
        3 => level3_path(),
        4 => level4_path(),
        5 => level5_path(),
        6 => level6_path(),
        7 => level7_path(),
        8 => level8_path(),
        9 => level9_path(),
        10 => level10_path(),
        _ => level1_path(),
    }
}

pub fn level_build_spots(level: u32) -> Vec<Vec3> {
    match level {
        1 => level1_build_spots(),
        2 => level2_build_spots(),
        3 => level3_build_spots(),
        4 => level4_build_spots(),
        5 => level5_build_spots(),
        6 => level6_build_spots(),
        7 => level7_build_spots(),
        8 => level8_build_spots(),
        9 => level9_build_spots(),
        10 => level10_build_spots(),
        _ => level1_build_spots(),
    }
}

pub fn level_waves(level: u32) -> Vec<WaveDefinition> {
    match level {
        1 => level1_waves(),
        2 => level2_waves(),
        3 => level3_waves(),
        4 => level4_waves(),
        5 => level5_waves(),
        6 => level6_waves(),
        7 => level7_waves(),
        8 => level8_waves(),
        9 => level9_waves(),
        10 => level10_waves(),
        _ => level1_waves(),
    }
}

pub fn level_theme(level: u32) -> ThemeColors {
    match level {
        1 | 2 => primordial_theme(),
        3 | 4 => prehistoric_theme(),
        5 | 6 => frozen_tundra_theme(),
        7 | 8 => ancient_theme(),
        9 | 10 => medieval_theme(),
        _ => primordial_theme(),
    }
}

pub fn level_hero_spawn(level: u32) -> Vec3 {
    match level {
        1 => hero_spawn_pos(),
        2 => level2_hero_spawn(),
        // Levels 3-10: spawn near path entrance (offset from first waypoint)
        _ => {
            let path = level_path(level);
            let first = path[0];
            Vec3::new(first.x, 0.0, first.z + 3.0)
        }
    }
}

pub struct LevelStartConfig {
    pub starting_gold: u32,
    pub lives: u32,
    pub max_waves: u32,
    pub wave_hp_scale: f32,
    pub wave_speed_scale: f32,
}

pub fn level_start_config(level: u32) -> LevelStartConfig {
    match level {
        1  => LevelStartConfig { starting_gold: 220, lives: 20, max_waves: 10, wave_hp_scale: 0.10, wave_speed_scale: 0.015 },
        2  => LevelStartConfig { starting_gold: 220, lives: 20, max_waves: 10, wave_hp_scale: 0.10, wave_speed_scale: 0.015 },
        3  => LevelStartConfig { starting_gold: 220, lives: 18, max_waves: 10, wave_hp_scale: 0.12, wave_speed_scale: 0.02 },
        4  => LevelStartConfig { starting_gold: 240, lives: 18, max_waves: 10, wave_hp_scale: 0.14, wave_speed_scale: 0.02 },
        5  => LevelStartConfig { starting_gold: 250, lives: 18, max_waves: 10, wave_hp_scale: 0.14, wave_speed_scale: 0.02 },
        6  => LevelStartConfig { starting_gold: 250, lives: 18, max_waves: 10, wave_hp_scale: 0.16, wave_speed_scale: 0.022 },
        7  => LevelStartConfig { starting_gold: 310, lives: 16, max_waves: 10, wave_hp_scale: 0.17, wave_speed_scale: 0.02 },
        8  => LevelStartConfig { starting_gold: 260, lives: 14, max_waves: 10, wave_hp_scale: 0.24, wave_speed_scale: 0.025 },
        9  => LevelStartConfig { starting_gold: 280, lives: 14, max_waves: 10, wave_hp_scale: 0.25, wave_speed_scale: 0.026 },
        10 => LevelStartConfig { starting_gold: 280, lives: 12, max_waves: 10, wave_hp_scale: 0.27, wave_speed_scale: 0.028 },
        _  => LevelStartConfig { starting_gold: 220, lives: 20, max_waves: 10, wave_hp_scale: 0.10, wave_speed_scale: 0.015 },
    }
}

/// Total number of levels available.
pub const MAX_LEVELS: u32 = 10;

pub struct LevelInfo {
    pub name: &'static str,
    pub era: &'static str,
    pub description: &'static str,
    pub waves: u32,
}

pub fn level_info(level: u32) -> LevelInfo {
    match level {
        1 => LevelInfo {
            name: "Primordial Pools", era: "Primordial Era",
            description: "Defend against ancient organisms in the murky swamplands.", waves: 10,
        },
        2 => LevelInfo {
            name: "Primordial Depths", era: "Primordial Era",
            description: "A twisting path through deeper waters. Beware the Giant Worm.", waves: 10,
        },
        3 => LevelInfo {
            name: "Jurassic Jungle", era: "Prehistoric Era",
            description: "Dinosaurs roam a lush jungle path.", waves: 10,
        },
        4 => LevelInfo {
            name: "Volcanic Pass", era: "Prehistoric Era",
            description: "A treacherous volcanic path. The T-Rex awaits.", waves: 10,
        },
        5 => LevelInfo {
            name: "Frozen Tundra", era: "Stone Age",
            description: "Ice age beasts cross the frozen wastes.", waves: 10,
        },
        6 => LevelInfo {
            name: "Glacier Gorge", era: "Stone Age",
            description: "A narrow gorge of ice. The Woolly Rhino charges.", waves: 10,
        },
        7 => LevelInfo {
            name: "Via Romana", era: "Ancient Era",
            description: "Legions march along the Roman road.", waves: 10,
        },
        8 => LevelInfo {
            name: "Colosseum", era: "Ancient Era",
            description: "Battle in the arena. Face the Minotaur.", waves: 10,
        },
        9 => LevelInfo {
            name: "Castle Approach", era: "Medieval Era",
            description: "Knights and cavalry assault the castle gates.", waves: 10,
        },
        10 => LevelInfo {
            name: "Dragon's Lair", era: "Medieval Era",
            description: "The final battle. Slay the Dragon.", waves: 10,
        },
        _ => LevelInfo {
            name: "Unknown", era: "Unknown", description: "", waves: 10,
        },
    }
}

pub struct HeroInfo {
    pub name: &'static str,
    pub role: &'static str,
    pub description: &'static str,
    pub color: [f32; 3],
}

pub fn hero_info(hero_type: HeroType) -> HeroInfo {
    match hero_type {
        HeroType::SacredMaiden => HeroInfo {
            name: "Aethon",
            role: "Balanced",
            description: "A holy warrior with healing and offensive abilities.",
            color: [1.0, 0.85, 0.4],
        },
        HeroType::IceHulk => HeroInfo {
            name: "Cryo",
            role: "Tank",
            description: "A massive frost giant that soaks damage and slows enemies.",
            color: [0.4, 0.7, 1.0],
        },
        HeroType::NorthernOutsider => HeroInfo {
            name: "Gorath",
            role: "Assassin",
            description: "A swift blade-dancer who excels at single-target damage.",
            color: [0.6, 0.9, 0.5],
        },
        HeroType::Pharaoh => HeroInfo {
            name: "Voltra",
            role: "Support",
            description: "An ancient ruler who commands fire and buffs allies.",
            color: [1.0, 0.6, 0.2],
        },
        HeroType::ScarletMagus => HeroInfo {
            name: "Ignis",
            role: "AoE Mage",
            description: "A powerful sorcerer specializing in area damage.",
            color: [1.0, 0.3, 0.3],
        },
    }
}

pub const ALL_HERO_TYPES: [HeroType; 5] = [
    HeroType::IceHulk,
    HeroType::NorthernOutsider,
    HeroType::Pharaoh,
    HeroType::ScarletMagus,
    HeroType::SacredMaiden, // Aethon — title-character capstone, last to unlock
];

/// Returns the level that must be beaten (stars > 0) to unlock a hero.
/// Players start with no hero (towers only) and earn each one as they progress.
pub fn hero_unlock_level(hero: HeroType) -> u32 {
    match hero {
        HeroType::IceHulk => 1,          // beat level 1
        HeroType::NorthernOutsider => 3, // beat level 3
        HeroType::Pharaoh => 5,          // beat level 5
        HeroType::ScarletMagus => 7,     // beat level 7
        HeroType::SacredMaiden => 9,     // beat level 9 — Aethon, the title hero
    }
}

pub struct EnemyInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub traits: &'static str,
}

pub fn enemy_info(enemy_type: EnemyType) -> EnemyInfo {
    match enemy_type {
        // Primordial
        EnemyType::Amoeba => EnemyInfo { name: "Amoeba", description: "Basic ground unit.", traits: "Ground" },
        EnemyType::Jellyfish => EnemyInfo { name: "Jellyfish", description: "Floats above the ground, bypassing blockers.", traits: "Flying" },
        EnemyType::Sporebloom => EnemyInfo { name: "Sporebloom", description: "Heals nearby allies over time.", traits: "Healer" },
        EnemyType::Trilobite => EnemyInfo { name: "Trilobite", description: "Slow but heavily armored.", traits: "Armored" },
        EnemyType::SeaScorpion => EnemyInfo { name: "Sea Scorpion", description: "Fast but fragile.", traits: "Fast" },
        EnemyType::Nautilus => EnemyInfo { name: "Armored Nautilus", description: "Extremely tough physical armor.", traits: "Heavy Armor" },
        EnemyType::GiantWorm => EnemyInfo { name: "Giant Worm", description: "A massive boss with high HP.", traits: "Boss" },
        // Prehistoric
        EnemyType::Raptor => EnemyInfo { name: "Raptor", description: "Lightning-fast predator.", traits: "Fast" },
        EnemyType::Stegosaurus => EnemyInfo { name: "Stegosaurus", description: "Sturdy herbivore.", traits: "Ground" },
        EnemyType::Parasaur => EnemyInfo { name: "Parasaurolophus", description: "A steady dinosaur.", traits: "Ground" },
        EnemyType::Triceratops => EnemyInfo { name: "Triceratops", description: "Heavily armored three-horned beast.", traits: "Heavy Armor" },
        EnemyType::Pterodactyl => EnemyInfo { name: "Pterodactyl", description: "Swoops overhead, ignoring blockers.", traits: "Flying" },
        EnemyType::CompyHealer => EnemyInfo { name: "Compy Healer", description: "Small dinosaur that heals allies.", traits: "Healer" },
        EnemyType::TRex => EnemyInfo { name: "Tyrannosaurus Rex", description: "King of the dinosaurs.", traits: "Boss" },
        // Stone Age
        EnemyType::Caveman => EnemyInfo { name: "Caveman", description: "Club-wielding warrior.", traits: "Ground" },
        EnemyType::Sabertooth => EnemyInfo { name: "Sabertooth", description: "Swift fanged predator.", traits: "Fast" },
        EnemyType::Mammoth => EnemyInfo { name: "Mammoth", description: "Massive tusked beast with thick hide.", traits: "Heavy Armor" },
        EnemyType::Shaman => EnemyInfo { name: "Shaman", description: "Tribal healer empowering nearby allies.", traits: "Healer" },
        EnemyType::GiantEagle => EnemyInfo { name: "Giant Eagle", description: "Soars above all defenses.", traits: "Flying" },
        EnemyType::Dodo => EnemyInfo { name: "Dodo", description: "Surprisingly fast flightless bird.", traits: "Fast" },
        EnemyType::WoollyRhino => EnemyInfo { name: "Woolly Rhino", description: "Unstoppable ice age titan.", traits: "Boss" },
        // Ancient
        EnemyType::Legionary => EnemyInfo { name: "Legionary", description: "Disciplined Roman soldier.", traits: "Ground" },
        EnemyType::Lion => EnemyInfo { name: "Lion", description: "Arena beast, swift and fierce.", traits: "Fast" },
        EnemyType::WarElephant => EnemyInfo { name: "War Elephant", description: "Armored war beast.", traits: "Heavy Armor" },
        EnemyType::EagleScout => EnemyInfo { name: "Eagle Scout", description: "Trained bird of prey.", traits: "Flying" },
        EnemyType::Medicus => EnemyInfo { name: "Medicus", description: "Roman field medic.", traits: "Healer" },
        EnemyType::Minotaur => EnemyInfo { name: "Minotaur", description: "Legendary labyrinth guardian.", traits: "Boss" },
        // Medieval
        EnemyType::Footman => EnemyInfo { name: "Footman", description: "Armored foot soldier.", traits: "Ground" },
        EnemyType::Cavalry => EnemyInfo { name: "Cavalry", description: "Mounted knight charging through.", traits: "Fast" },
        EnemyType::Knight => EnemyInfo { name: "Knight", description: "Heavily armored elite warrior.", traits: "Heavy Armor" },
        EnemyType::Wyvern => EnemyInfo { name: "Wyvern", description: "Dragon-kin soaring overhead.", traits: "Flying" },
        EnemyType::Priest => EnemyInfo { name: "Priest", description: "Holy healer blessing nearby troops.", traits: "Healer" },
        EnemyType::Dragon => EnemyInfo { name: "Dragon", description: "The ultimate beast. Fire and fury.", traits: "Boss" },
    }
}

/// Returns true for enemy types that serve as wave-10 bosses.
pub fn is_boss_type(enemy_type: EnemyType) -> bool {
    matches!(
        enemy_type,
        EnemyType::GiantWorm
            | EnemyType::TRex
            | EnemyType::WoollyRhino
            | EnemyType::Minotaur
            | EnemyType::Dragon
    )
}

pub const ALL_ENEMY_TYPES: [EnemyType; 33] = [
    // Primordial
    EnemyType::Amoeba, EnemyType::Jellyfish, EnemyType::Sporebloom,
    EnemyType::Trilobite, EnemyType::SeaScorpion, EnemyType::Nautilus, EnemyType::GiantWorm,
    // Prehistoric
    EnemyType::Raptor, EnemyType::Stegosaurus, EnemyType::Parasaur,
    EnemyType::Triceratops, EnemyType::Pterodactyl, EnemyType::CompyHealer, EnemyType::TRex,
    // Stone Age
    EnemyType::Caveman, EnemyType::Sabertooth, EnemyType::Mammoth,
    EnemyType::Shaman, EnemyType::GiantEagle, EnemyType::Dodo, EnemyType::WoollyRhino,
    // Ancient
    EnemyType::Legionary, EnemyType::Lion, EnemyType::WarElephant,
    EnemyType::EagleScout, EnemyType::Medicus, EnemyType::Minotaur,
    // Medieval
    EnemyType::Footman, EnemyType::Cavalry, EnemyType::Knight,
    EnemyType::Wyvern, EnemyType::Priest, EnemyType::Dragon,
];

// ---------------------------------------------------------------------------
// Hero abilities
// ---------------------------------------------------------------------------

pub struct AbilityDef {
    pub name: &'static str,
    pub cooldown: f32,
    pub effect: AbilityEffect,
    pub color: [f32; 3],
}

// ---------------------------------------------------------------------------
// Global player abilities (meteor & reinforcements)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAbilityType {
    Meteor,
    Reinforcements,
}

pub struct PlayerAbilityDef {
    pub name: &'static str,
    pub description: &'static str,
    pub cooldown: f32,
    pub icon_color: [f32; 3],
}

pub fn player_ability_def(ability: PlayerAbilityType) -> PlayerAbilityDef {
    match ability {
        PlayerAbilityType::Meteor => PlayerAbilityDef {
            name: "Meteor",
            description: "Massive AoE damage",
            cooldown: 30.0,
            icon_color: [1.0, 0.4, 0.1],
        },
        PlayerAbilityType::Reinforcements => PlayerAbilityDef {
            name: "Reinforcements",
            description: "Spawn soldiers",
            cooldown: 40.0,
            icon_color: [0.3, 0.7, 0.3],
        },
    }
}

/// Meteor stats
pub const METEOR_DAMAGE: f32 = 200.0;
pub const METEOR_RADIUS: f32 = 4.0;

/// Reinforcements stats
pub const REINFORCEMENT_COUNT: u32 = 2;
pub const REINFORCEMENT_HP: f32 = 150.0;
pub const REINFORCEMENT_DAMAGE: f32 = 8.0;
pub const REINFORCEMENT_DURATION: f32 = 15.0;

#[derive(Clone, Copy, Debug)]
pub enum AbilityEffect {
    /// Deal damage to enemies in radius around hero.
    AoeDamage { damage: f32, radius: f32 },
    /// Heal hero by percentage of max HP.
    Heal { percent: f32 },
    /// Slow enemies in radius around hero.
    AoeSlow { factor: f32, duration: f32, radius: f32 },
    /// Deal damage + apply slow in radius.
    AoeDamageAndSlow { damage: f32, radius: f32, slow_factor: f32, slow_duration: f32 },
    /// Deal damage + apply burn in radius.
    AoeDamageAndBurn { damage: f32, radius: f32, burn_dps: f32, burn_duration: f32 },
    /// Deal multiplied damage to nearest enemy.
    SingleTargetBurst { multiplier: f32, range: f32 },
    /// Reduce incoming damage for duration.
    DamageReduction { factor: f32, duration: f32 },
}

/// Returns the 3 abilities for a given hero type.
pub fn hero_abilities(hero_type: HeroType) -> [AbilityDef; 3] {
    match hero_type {
        HeroType::SacredMaiden => [
            AbilityDef {
                name: "Holy Strike", cooldown: 15.0,
                effect: AbilityEffect::AoeDamage { damage: 40.0, radius: 3.5 },
                color: [1.0, 0.9, 0.4],
            },
            AbilityDef {
                name: "Divine Blessing", cooldown: 25.0,
                effect: AbilityEffect::Heal { percent: 0.6 },
                color: [0.3, 1.0, 0.4],
            },
            AbilityDef {
                name: "Sacred Ground", cooldown: 20.0,
                effect: AbilityEffect::AoeSlow { factor: 0.5, duration: 3.0, radius: 4.0 },
                color: [0.6, 0.8, 1.0],
            },
        ],
        HeroType::IceHulk => [
            AbilityDef {
                name: "Ground Slam", cooldown: 12.0,
                effect: AbilityEffect::AoeDamage { damage: 60.0, radius: 3.0 },
                color: [0.7, 0.5, 0.3],
            },
            AbilityDef {
                name: "Frost Armor", cooldown: 30.0,
                effect: AbilityEffect::DamageReduction { factor: 0.5, duration: 5.0 },
                color: [0.3, 0.6, 1.0],
            },
            AbilityDef {
                name: "Frost Nova", cooldown: 25.0,
                effect: AbilityEffect::AoeDamageAndSlow { damage: 40.0, radius: 5.0, slow_factor: 0.3, slow_duration: 3.0 },
                color: [0.5, 0.9, 1.0],
            },
        ],
        HeroType::NorthernOutsider => [
            AbilityDef {
                name: "Blade Fury", cooldown: 8.0,
                effect: AbilityEffect::SingleTargetBurst { multiplier: 3.0, range: 3.0 },
                color: [1.0, 0.3, 0.3],
            },
            AbilityDef {
                name: "Whirlwind", cooldown: 12.0,
                effect: AbilityEffect::AoeDamage { damage: 30.0, radius: 3.0 },
                color: [0.8, 0.8, 0.8],
            },
            AbilityDef {
                name: "Execute", cooldown: 18.0,
                effect: AbilityEffect::SingleTargetBurst { multiplier: 5.0, range: 2.5 },
                color: [0.8, 0.1, 0.1],
            },
        ],
        HeroType::Pharaoh => [
            AbilityDef {
                name: "Sandstorm", cooldown: 18.0,
                effect: AbilityEffect::AoeDamageAndSlow { damage: 25.0, radius: 4.0, slow_factor: 0.4, slow_duration: 3.0 },
                color: [0.9, 0.7, 0.3],
            },
            AbilityDef {
                name: "Sun's Wrath", cooldown: 30.0,
                effect: AbilityEffect::AoeDamage { damage: 80.0, radius: 5.0 },
                color: [1.0, 0.85, 0.0],
            },
            AbilityDef {
                name: "Blessing of Ra", cooldown: 22.0,
                effect: AbilityEffect::Heal { percent: 0.5 },
                color: [0.4, 1.0, 0.6],
            },
        ],
        HeroType::ScarletMagus => [
            AbilityDef {
                name: "Fireball", cooldown: 10.0,
                effect: AbilityEffect::AoeDamageAndBurn { damage: 50.0, radius: 2.5, burn_dps: 5.0, burn_duration: 3.0 },
                color: [1.0, 0.4, 0.1],
            },
            AbilityDef {
                name: "Flame Wave", cooldown: 14.0,
                effect: AbilityEffect::AoeDamage { damage: 45.0, radius: 4.0 },
                color: [1.0, 0.6, 0.2],
            },
            AbilityDef {
                name: "Inferno", cooldown: 45.0,
                effect: AbilityEffect::AoeDamage { damage: 100.0, radius: 6.0 },
                color: [1.0, 0.2, 0.0],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Meta-progression upgrades
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpgradeKind {
    WarChest,         // +10% starting gold per level
    ElementalFury,    // +5% tower damage per level
    FarSight,         // +5% tower range per level
    SalvageExpert,    // +5% sell refund per level
    TacticalMastery,  // -5% hero ability cooldowns per level
}

pub const ALL_UPGRADES: [UpgradeKind; 5] = [
    UpgradeKind::WarChest,
    UpgradeKind::ElementalFury,
    UpgradeKind::FarSight,
    UpgradeKind::SalvageExpert,
    UpgradeKind::TacticalMastery,
];

pub const UPGRADE_MAX_LEVEL: u8 = 3;
pub const UPGRADE_COSTS: [u32; 3] = [20, 50, 100];

pub struct UpgradeDef {
    pub name: &'static str,
    pub description: &'static str,
    pub per_level: &'static str,
}

pub fn upgrade_def(kind: UpgradeKind) -> UpgradeDef {
    match kind {
        UpgradeKind::WarChest => UpgradeDef {
            name: "War Chest",
            description: "Start each level with more gold.",
            per_level: "+10% starting gold",
        },
        UpgradeKind::ElementalFury => UpgradeDef {
            name: "Elemental Fury",
            description: "All towers deal more damage.",
            per_level: "+5% tower damage",
        },
        UpgradeKind::FarSight => UpgradeDef {
            name: "Far Sight",
            description: "All towers have increased range.",
            per_level: "+5% tower range",
        },
        UpgradeKind::SalvageExpert => UpgradeDef {
            name: "Salvage Expert",
            description: "Selling towers returns more gold.",
            per_level: "+5% sell refund",
        },
        UpgradeKind::TacticalMastery => UpgradeDef {
            name: "Tactical Mastery",
            description: "Hero abilities recharge faster.",
            per_level: "-5% cooldowns",
        },
    }
}

/// Returns the upgrade index (0-4) for a given UpgradeKind.
pub fn upgrade_index(kind: UpgradeKind) -> usize {
    match kind {
        UpgradeKind::WarChest => 0,
        UpgradeKind::ElementalFury => 1,
        UpgradeKind::FarSight => 2,
        UpgradeKind::SalvageExpert => 3,
        UpgradeKind::TacticalMastery => 4,
    }
}
