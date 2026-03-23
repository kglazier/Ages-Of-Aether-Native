# Session Notes — 2026-03-20

## What Was Done

### Hero Idle & Attack Animations (WORKING)
- Sacred Maiden GLB has 0 embedded animations. Bevy doesn't auto-create AnimationPlayer/AnimationTarget for such models.
- Fix: manually insert `AnimationPlayer::default()` on the Armature entity, then recursively insert `AnimationTarget` on every bone using `AnimationTargetId::from_names(path)`.
- Idle (`maiden-idle.glb`) and attack (`maiden-melee-kick.glb`) clips load from separate files and play correctly.
- Sacred Maiden model has TWO skeleton hierarchies: one with numeric suffixes (nodes 0-56), one clean Mixamo (nodes 58-100, rooted at `Armature`). Our code attaches to the clean one.

### Hero Attack Facing (WORKING)
- Model's visual forward is +Z. Bevy's `look_at` points -Z toward target.
- Fix: `hero_tf.look_at(enemy_pos, Vec3::Y); hero_tf.rotate_y(PI);`
- User confirmed this is correct — do NOT change it.

### Attack Animation Glitch (IN PROGRESS)
- Original issue: `.repeat()` on a short kick clip caused rapid-fire restarts.
- Attempt 1: Fixed 0.8s timer to control attack duration — still glitchy because timer/cooldown mismatch caused brief idle flashes.
- Current approach: play attack clip ONCE per `attack_triggered` flag (set by `hero_auto_attack` in Combat set). Hero stays in Attack state while enemies are in range. Clip plays through fully, holds last frame, restarts only when next attack fires. **Not yet tested by user.**

### Hero Walk/Run Animation (NOT WORKING)
- Added `run_anim` field to `HeroStats`, set to `models/enemies/anims/run.glb` for all heroes.
- Loaded as `walk_node` in the AnimationGraph. `update_hero_animations` plays it during `HeroAnimKind::Walk`.
- **Bone hierarchy verified matching**: Both `run.glb` and `maiden-idle.glb` use identical paths (`Armature → mixamorig:Hips → mixamorig:Spine → ...`). AnimationTargetIds should match.
- **Still not working visually** — hero shows no animation during movement.
- Possible causes to investigate:
  - Asset loading timing (clip might not be loaded when first played)
  - Subtle difference in how Bevy's GLTF loader computes AnimationTargetId vs our manual insertion
  - Try loading `walk.glb` or `maiden-idle.glb` at higher speed as diagnostic
  - Try using the attack clip as walk (diagnostic: confirms walk state fires but clip is the issue)
  - Check `adb logcat` for "Hero switching to WALK/RUN animation" log to confirm state transition fires

### Hero Movement Direction (UNCERTAIN)
- User said sliding is related to the run animation — once run animation works, sliding goes away.
- `hero_movement` currently uses `look_at` WITHOUT `rotate_y(PI)`. Was reverted per user request.
- May need `rotate_y(PI)` added back once run animation works (same pattern as attack facing).

### Hero Blocking Enemies
- `hero_block_enemies` moved from `GameSet::Combat` to `GameSet::Input` (runs before movement, like golem blocking).
- `hero_unblock_enemies` moved to `GameSet::Cleanup`.
- Uses shared `GolemBlocked` component (same as golems).
- `spread_blocked_enemies` uses `Local<HashSet<Entity>>` for deterministic fan placement.
- User reported enemies still sliding past hero — may need further debugging.

### GameWorldEntity Cleanup (WORKING)
- Added `GameWorldEntity` to enemies (wave.rs) and towers (ui.rs).
- Restart now properly clears the game world.

## Key Files Modified
- `src/data.rs` — Added `run_anim` field to `HeroStats`
- `src/systems/hero.rs` — Hero animation system (setup, play, update), attack trigger, blocking
- `src/systems/mod.rs` — System set assignments (hero_block_enemies → Input, hero_unblock_enemies → Cleanup)
- `src/systems/path.rs` — Blocked enemy headbutt animation (scale bob, no XZ drift)
- `src/systems/golem.rs` — `spread_blocked_enemies` deterministic fan pattern
- `src/components.rs` — `HeroNeedsAnimation` marker component

## Completed (Session 2 — 2026-03-21)

### Hero Animations (ALL WORKING)
- **Run animation** — Stripped Hips root motion curves from run clip via `curves_mut().remove()`. Hero no longer drifts from selection circle.
- **Attack animation** — Kick plays correctly, only strips run clip (not attack/idle).
- **Hero facing** — `rotate_y(PI)` in both `hero_movement` and `face_enemy` for correct facing during run and attack.
- **Auto-attack** — Skips attack while hero is moving (prevents animation interruption).

### Unified Blocking System (WORKING)
- Rewrote to match original Three.js approach: one `block_enemies` system handles both hero (1.5 range) and golems (1.8 range).
- Random spread offset applied once when enemy first blocked (angle + distance 0.3-0.8), removed on unblock.
- Removed old separate `hero_block_enemies`, `hero_unblock_enemies`, `golem_block_enemies`, `spread_blocked_enemies`, `advance_blocked_enemies`.
- `golem_assign_targets` — stripped down to just pick which enemy each golem faces/attacks.
- All blocked enemies deal damage to blockers (hero and golems).

### Golem Respawn Timer (WORKING)
- 12s respawn delay matching original Three.js system. Tower gets `GolemRespawnTimer` component on golem death.

### Hero Death Unblocking (WORKING)
- Enemies unblock when hero dies (hero filtered by `Without<HeroRespawnTimer>`).

## Completed (Session 3 — 2026-03-21)

### Enemy Skeletal Animations (DONE)
- New `EnemyAnimState` component tracks walk/idle/attack/death animation nodes per enemy.
- `EnemyNeedsAnimation` marker added to spawned enemies; `setup_enemy_animations` finds AnimationPlayer in loaded GLTF scene, builds AnimationGraph from `anim_indices`.
- Walk animation plays by default, switches to attack when blocked, death animation on kill.
- Enemies with skeletal animations skip procedural bob/squash-stretch in `move_enemies`.
- Death animation: enemies get `EnemyDying` component (1.2s timer) instead of immediate despawn. `Enemy` component removed so they stop being targeted/blocked.
- Files: `src/systems/enemy_anim.rs` (new), `src/components.rs`, `src/systems/wave.rs`, `src/systems/path.rs`, `src/systems/combat.rs`

### Hero Abilities (DONE)
- 3 unique abilities per hero defined in `data.rs` via `hero_abilities()`.
- Effects: AoeDamage, Heal, AoeSlow, AoeDamageAndSlow, AoeDamageAndBurn, SingleTargetBurst, DamageReduction.
- `HeroAbilities` component tracks cooldowns per ability.
- `HeroDamageReduction` buff component reduces incoming damage (used by Frost Armor).
- `AbilityActivated` resource bridges UI button press → ability execution.
- Ability VFX: expanding ring at hero position using reused `DeathEffect`.
- Hero HUD updated: 3 colored ability buttons with name + cooldown text. Buttons dim when on cooldown.
- Files: `src/systems/hero_ability.rs` (new), `src/data.rs`, `src/components.rs`, `src/ui.rs`, `src/systems/hero.rs`

### Elemental Synergies (DONE)
- **Ice+Lightning**: Slowed enemies take +50% bonus true damage (ignores armor) from lightning projectiles.
- **Ice+Fire**: Slowed enemies take 40% of fire damage as bonus true damage.
- **Earth+Ice**: Golems within 8 units of an ice tower apply slow (50%, 1.5s) to blocked enemies.
- **Earth+Fire**: Golems within 8 units of a fire tower apply burn (2 dps, 1.5s) to blocked enemies.
- Synergy checks in `move_projectiles` (damage synergies) and `golem_elemental_synergy` (proximity aura).
- Files: `src/systems/combat.rs`

## Completed (Session 4 — 2026-03-21)

### Northern Outsider Animations (WORKING)
- **Problem**: Model's bones have ~100x larger translations than generic Mixamo animations expect (Hips at [1118.85, -5036.71, -8780.71] vs animation's [0.0, 0.0, -104.3]). Applying animations caused garbled mesh.
- **Failed approach**: Bind-pose reset (capturing bind-pose translations/scales and resetting every frame in PostUpdate). Models were either "standing still" or "jumbled messes".
- **Working solution**: **Runtime curve stripping**. GLTF exports curves per bone in deterministic order: `[translation=0, rotation=1, scale=2]`. The `strip_hero_rotation_only_clips` system waits for all clips to load, then removes translation and scale curves, keeping only rotation (index 1). This is the Bevy equivalent of Three.js stripping `.position`/`.scale` tracks.
- **Key flags**: `rotation_only_anims: true` + `skip_root_motion_cancel: true` in HeroStats
- **Key system**: `hero.rs::strip_hero_rotation_only_clips` (runs in GameSet::Spawning)
- **Animation files**: mutant-idle.glb (idle), melee-combo.glb (attack), run.glb (run)
- **Note**: outsider-melee.glb (character-specific attack) did NOT work — different bone paths

### Pharaoh Orientation Fix (WORKING)
- **Problem**: Model exported with wrong up-axis (Z-up instead of Y-up), rendering on its back.
- **Failed approach**: Rotating the SceneRoot parent — worked for idle but run/attack animations overwrote the orientation.
- **Working solution**: PostUpdate system sets Armature entity rotation to `Quat::from_rotation_x(FRAC_PI_2)` every frame after `animate_targets`. Must SET (not multiply) to prevent compounding.
- **Key fields in HeroAnimState**: `armature_entity: Option<Entity>`, `armature_rotation_fix: Option<Quat>`
- **Code**: `cancel_hero_root_motion` in PostUpdate applies the fix
- **Important**: The name check in `setup_hero_animations` must happen BEFORE the `break` when finding AnimationPlayer, otherwise `armature_entity` stays None

### Hero Model Configuration
| Hero | Scale | Rot X | Y Offset | Idle | Attack | Special |
|------|-------|-------|----------|------|--------|---------|
| Sacred Maiden | 1.0 | 0 | 0 | maiden-idle | maiden-melee-kick | — |
| Ice Hulk | 1.5 | 0 | 0 | mutant-idle | melee-combo | — |
| Northern Outsider | 0.009 | 0 | 0 | mutant-idle | melee-combo | rotation_only + skip_root_motion |
| Pharaoh | 0.015 | +90° | 1.5 | maiden-idle | maiden-melee-kick | armature rotation fix |
| Scarlet Magus | 0.015 | 0 | 1.0 | maiden-idle | maiden-melee-kick | — |

All heroes use `models/enemies/anims/run.glb` for run animation.

### Attack Animation Playback Fix (WORKING)
- **Problem**: Attack animation restarted before completing because `hero_auto_attack` cooldown fired and set `attack_triggered`, which immediately restarted the clip.
- **Fix**: In `update_hero_animations`, the "is attack mid-play" check (`all_finished()`) now runs BEFORE the `attack_triggered` processing. If the attack is still playing, the trigger is preserved for later and the animation continues uninterrupted.

### Pause Menu Quit & Restart Confirm (WORKING)
- Added **Quit** button to pause screen → goes to Level Select
- Both **Restart** and **Quit** now show confirmation dialog ("Restart this level?" / "Quit to level select?" with Yes/No)
- Quit manually cleans up GameWorldEntity + HudRoot + HeroHudRoot before transitioning
- Restart just resets resources and goes to WaitingForWindow (lets OnEnter(Playing) handle cleanup)
- ConfirmDialog entity cleaned up in `cleanup_pause_screen` on state exit
- Components: `ConfirmDialog`, `ConfirmYesButton`, `ConfirmNoButton`, `PendingConfirm` resource

### Hero Select Screen (MOBILE FIX)
- Reduced hero card width from 200px to 150px, smaller fonts
- Cards container has `overflow: Overflow::scroll_y()` and `flex_grow: 1.0`
- Back + Start buttons pinned to bottom with `justify_content: JustifyContent::SpaceBetween` on root
- All content fits on mobile landscape without scrolling (but scrollable if needed)

### Debug Screen Updates
- Switched from bind-pose reset to curve stripping approach (`NeedsCurveStrip` component)
- `strip_debug_rotation_only_clips` system strips curves on debug screen models
- Last configuration: Pharaoh + Scarlet Magus rotation variants for testing

## Remaining Tasks

### High Priority
1. **Tower specializations** — 2 per element, choice at max level
2. **More levels** (Level 2-5)

### Medium Priority
3. Save/load system
4. Meta-progression upgrade shop
5. Android build optimization & deploy test

## Technical Notes

### Bevy Animation Retargeting (for next session)
- `AnimationTargetId` is a Blake3 hash of the bone name path from root to target.
- When loading a clip from a separate GLB, Bevy computes target IDs from that file's node hierarchy.
- For cross-file retargeting to work, the bone paths must produce identical hashes.
- Our manual `insert_anim_targets_recursive` builds paths from entity `Name` components.
- The paths SHOULD match (verified by inspecting both GLB node hierarchies), but the animation doesn't play. This is the key mystery to solve.

### System Ordering
```
Input:    handle_world_click, hero_consume_move_command, block_enemies, golem_assign_targets
Spawning: wave_spawner, spawn_golems, setup_hero_animations, play_hero_animations
Movement: move_enemies, hero_movement, update_hero_animations, golem_movement
Combat:   hero_auto_attack, tower_targeting, enemies_attack_hero, enemies_attack_golem, golem_melee_attack
Cleanup:  check_enemy_death, check_golem_death, hero_death_check, hero_respawn_tick
Visual:   update_health_bars, update_hero_visuals
```
