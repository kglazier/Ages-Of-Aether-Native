use bevy::prelude::*;
use crate::data::EnemyType;

// ---------------------------------------------------------------------------
// Common
// ---------------------------------------------------------------------------

/// Marker for all entities that belong to the game world and should be cleaned up on restart.
#[derive(Component)]
pub struct GameWorldEntity;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Element {
    Lightning,
    Earth,
    Ice,
    Fire,
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::Lightning => write!(f, "Lightning"),
            Element::Earth => write!(f, "Earth"),
            Element::Ice => write!(f, "Ice"),
            Element::Fire => write!(f, "Fire"),
        }
    }
}

// ---------------------------------------------------------------------------
// Enemy
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyTypeId(pub EnemyType);

/// Marks an enemy whose materials need recoloring after scene load.
#[derive(Component)]
pub struct EnemyNeedsTint(pub Color);

#[derive(Component)]
pub struct PathFollower {
    pub segment: usize,
    pub progress: f32,
    pub speed: f32,
    pub base_speed: f32,
    /// Lateral offset perpendicular to path direction for visual spacing.
    pub lateral_offset: f32,
    /// Vertical offset to keep model above ground (varies by enemy type).
    pub y_offset: f32,
}

#[derive(Component)]
pub struct GoldReward(pub u32);

#[derive(Component)]
pub struct Armor {
    pub physical: f32,
    pub magic_resist: f32,
}

/// Slowed: speed reduced by factor (0.5 = half speed) for `remaining` seconds.
#[derive(Component)]
pub struct SlowDebuff {
    pub factor: f32,
    pub remaining: f32,
}

/// Burning: takes `dps` damage per second for `remaining` seconds.
#[derive(Component)]
pub struct BurnDebuff {
    pub dps: f32,
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Tower
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Tower;

#[derive(Component)]
pub struct TowerLevel(pub u8);

/// Total gold invested in this tower (for sell refund calculation).
#[derive(Component)]
pub struct TowerInvestment(pub u32);

/// Links tower back to its build spot for sell/cleanup.
#[derive(Component)]
pub struct BuildSpotRef(pub Entity);

/// Persists the player-set rally point on the tower (survives golem death/respawn).
#[derive(Component)]
pub struct TowerRallyPoint(pub Vec3);

#[derive(Component)]
pub struct AttackTimer {
    pub cooldown: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct AttackRange(pub f32);

#[derive(Component)]
pub struct AttackDamage(pub f32);

// ---------------------------------------------------------------------------
// Build spot
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct BuildSpot {
    pub id: usize,
    pub occupied: bool,
}

// ---------------------------------------------------------------------------
// Projectile
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub target: Entity,
    pub element: Element,
}

/// Fire projectiles splash damage in an area on impact.
#[derive(Component)]
pub struct AoeSplash(pub f32);

// ---------------------------------------------------------------------------
// Golem (spawned by Earth towers)
// ---------------------------------------------------------------------------

/// Marker: this entity is a golem.
#[derive(Component)]
pub struct Golem;

/// Which tower owns this golem.
#[derive(Component)]
pub struct GolemOwner(pub Entity);

/// Where the golem should stand to block enemies.
#[derive(Component)]
pub struct GolemRallyPoint(pub Vec3);

/// The enemy this golem is currently blocking (if any).
#[derive(Component)]
pub struct BlockingEnemy(pub Option<Entity>);

/// Golem melee attack timer.
#[derive(Component)]
pub struct GolemAttack {
    pub damage: f32,
    pub cooldown: f32,
    pub elapsed: f32,
}

/// Marker: this enemy is currently blocked by a golem/hero and cannot move.
#[derive(Component)]
pub struct GolemBlocked;

/// Random spread offset applied when an enemy first becomes blocked.
/// Removed when unblocked.
#[derive(Component)]
pub struct BlockOffset(pub Vec3);

/// Marker: enemy was just unblocked and needs path scatter to avoid blobbing.
#[derive(Component)]
pub struct NeedsUnblockScatter(pub u32);

// ---------------------------------------------------------------------------
// Death effects
// ---------------------------------------------------------------------------

/// Visual burst spawned when an enemy dies.
#[derive(Component)]
pub struct DeathEffect {
    pub lifetime: f32,
    pub elapsed: f32,
}

/// Floating gold indicator that rises and fades.
#[derive(Component)]
pub struct GoldPopup {
    pub lifetime: f32,
    pub elapsed: f32,
    pub start_y: f32,
}

/// Brief muzzle flash when a tower fires.
#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: f32,
    pub elapsed: f32,
    pub element: Element,
}

// ---------------------------------------------------------------------------
// Range indicator
// ---------------------------------------------------------------------------

/// 3D range circle shown when a tower is selected.
#[derive(Component)]
pub struct RangeIndicator;

// ---------------------------------------------------------------------------
// Flying
// ---------------------------------------------------------------------------

/// Marker: this enemy flies above the ground and can't be blocked by golems.
#[derive(Component)]
pub struct Flying;

/// Original model scale for procedural animation (squash/stretch).
#[derive(Component)]
pub struct ModelScale(pub f32);

// ---------------------------------------------------------------------------
// Healer
// ---------------------------------------------------------------------------

/// Healer aura: heals nearby enemies within radius.
#[derive(Component)]
pub struct HealerAura {
    pub radius: f32,
    pub heal_per_second: f32,
}

/// Visual ring under healer enemies. Tracks which enemy it belongs to.
#[derive(Component)]
pub struct HealerRing(pub Entity);

// ---------------------------------------------------------------------------
// Upgrade indicators
// ---------------------------------------------------------------------------

/// Small visual markers showing tower upgrade level.
#[derive(Component)]
pub struct UpgradeIndicator {
    pub tower: Entity,
}

/// Tracks the last known level so we know when to refresh indicators.
#[derive(Component)]
pub struct LastKnownLevel(pub u8);

// ---------------------------------------------------------------------------
// Tower specializations
// ---------------------------------------------------------------------------

/// Marks a tower as having been specialized.
#[derive(Component, Clone, Copy)]
pub struct TowerSpec(pub crate::data::TowerSpecialization);

/// Propagated onto projectiles from specialized towers.
#[derive(Component, Clone, Copy)]
pub struct ProjectileSpec(pub crate::data::TowerSpecialization);

/// Aura effect from Bramble Grove or Blizzard Tower.
#[derive(Component)]
pub struct TowerAura {
    pub tower: Entity,
    pub radius: f32,
    pub slow_factor: Option<f32>,
    pub dps: Option<f32>,
}

/// Burning ground zone from Inferno Cannon impacts.
#[derive(Component)]
pub struct BurnZone {
    pub radius: f32,
    pub dps: f32,
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Hero
// ---------------------------------------------------------------------------

/// Marker: this entity is the player's hero.
#[derive(Component)]
pub struct Hero;

/// Visual Y offset for hero model (applied to scene child, not root entity).
/// Keeps root entity at ground level for accurate blocking/distance checks.
#[derive(Component)]
pub struct HeroModelYOffset(pub f32);

/// Where the hero is moving toward (None = standing still).
#[derive(Component)]
pub struct HeroMoveTarget(pub Option<Vec3>);

/// Hero's melee attack timer.
#[derive(Component)]
pub struct HeroAttackTimer {
    pub cooldown: f32,
    pub elapsed: f32,
}

/// Hero's attack range.
#[derive(Component)]
pub struct HeroAttackRange(pub f32);

/// Hero's attack damage.
#[derive(Component)]
pub struct HeroAttackDamage(pub f32);

/// Hero's movement speed.
#[derive(Component)]
pub struct HeroMoveSpeed(pub f32);

/// When present, hero is dead and respawning after `remaining` seconds.
#[derive(Component)]
pub struct HeroRespawnTimer {
    pub remaining: f32,
    pub total: f32,
    /// Where the hero died — respawn here instead of fixed spawn point.
    pub death_pos: Vec3,
}

/// 3D health bar fill that follows the hero.
#[derive(Component)]
pub struct HeroHealthBar3d;

/// 3D health bar background for the hero.
#[derive(Component)]
pub struct HeroHealthBarBg3d;

/// Glowing selection ring on ground under the hero.
#[derive(Component)]
pub struct HeroSelectionRing;

/// Visual marker showing where the hero is moving to.
#[derive(Component)]
pub struct HeroMoveMarker;

/// Marker for heroes that need their animation set up after scene loads.
#[derive(Component)]
pub struct HeroNeedsAnimation;

// ---------------------------------------------------------------------------
// Hero abilities
// ---------------------------------------------------------------------------

/// Tracks cooldowns for the hero's 3 abilities.
#[derive(Component)]
pub struct HeroAbilities {
    pub cooldowns: [f32; 3],
    pub max_cooldowns: [f32; 3],
}

/// Temporary damage reduction buff on the hero.
#[derive(Component)]
pub struct HeroDamageReduction {
    pub factor: f32,
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Enemy animations
// ---------------------------------------------------------------------------

/// Marker for enemies that need their animation set up after scene loads.
#[derive(Component)]
pub struct EnemyNeedsAnimation;

/// Tracks which animation is playing for an enemy and stores graph node indices.
#[derive(Component)]
pub struct EnemyAnimState {
    pub walk_node: AnimationNodeIndex,
    pub idle_node: AnimationNodeIndex,
    pub attack_node: AnimationNodeIndex,
    pub death_node: AnimationNodeIndex,
    pub current: EnemyAnimKind,
    pub player_entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyAnimKind {
    Walk,
    Idle,
    Attack,
    Death,
}

/// Procedural walk animation for non-humanoid enemies without embedded animations.
#[derive(Component)]
pub struct ProceduralWalkAnim {
    pub phase: f32,
}

/// Stores discovered quadruped leg bone entities for programmatic walk animation.
#[derive(Component)]
pub struct QuadLegBones {
    /// Leg bones: (entity, phase_offset, bind_euler_z, bind_euler_y).
    /// Uses ZYX Euler order so X (outermost) = parent-axis swing, matching Three.js.
    pub legs: Vec<(Entity, f32, f32, f32)>,
    /// Foot IK-target bones: (entity, phase_offset, bind_quaternion, bind_translation).
    /// Translated vertically to follow leg swing arc.
    pub feet: Vec<(Entity, f32, Quat, Vec3)>,
}

/// Marker: leg bones haven't been discovered yet for this procedural-walk enemy.
#[derive(Component)]
pub struct NeedsLegDiscovery;

/// Enemy is dying — plays death animation then despawns after timer.
#[derive(Component)]
pub struct EnemyDying {
    pub timer: f32,
}

// ---------------------------------------------------------------------------
// Health bars
// ---------------------------------------------------------------------------

/// HP bar fill that tracks an enemy and scales with health percentage.
#[derive(Component)]
pub struct HealthBar(pub Entity);

/// HP bar dark background behind the fill.
#[derive(Component)]
pub struct HealthBarBg(pub Entity);

// ---------------------------------------------------------------------------
// Damage numbers
// ---------------------------------------------------------------------------

/// Tracks last known health to detect damage for floating numbers.
#[derive(Component)]
pub struct LastHealth(pub f32);

/// Floating damage number that rises and fades.
#[derive(Component)]
pub struct DamageNumber {
    pub lifetime: f32,
    pub elapsed: f32,
    pub start_y: f32,
}

// ---------------------------------------------------------------------------
// Placement / upgrade animations
// ---------------------------------------------------------------------------

/// Drives an easeOutBack scale bounce when a tower is placed or upgraded.
#[derive(Component)]
pub struct PlacementBounce {
    pub duration: f32,
    pub elapsed: f32,
    pub target_scale: f32,
}

/// Brief emissive flash after upgrading a tower.
#[derive(Component)]
pub struct UpgradeFlash {
    pub remaining: f32,
}
