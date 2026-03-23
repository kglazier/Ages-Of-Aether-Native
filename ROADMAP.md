# Ages of Aether — Native Port Roadmap

Porting from TypeScript/Three.js/Capacitor to Rust/Bevy for Android.
Primary motivation: eliminate V8 garbage collection pauses on mobile.

---

## Legend
- [x] Done in native port
- [ ] Not yet ported / needs implementation
- (OLD) = existed in TS version, needs porting
- (NEW) = new for native version

---

## Core Loop (Phase 1)

- [x] Bevy app setup, window, camera, lighting
- [x] Ground plane + path rendering from waypoints
- [x] Build spots (clickable positions)
- [x] Enemy spawning and waypoint movement
- [x] Tower placement (click build spot → select element)
- [x] Tower targeting (nearest enemy in range)
- [x] Projectile system (spawn, fly, hit, despawn)
- [x] Damage + enemy death
- [x] Gold system (earn on kill, spend to build)
- [x] Lives system (enemy reaches end, lose a life)
- [x] Basic HUD (gold, lives, wave counter)
- [x] Win/lose conditions + Game Over screen
- [x] Play Again button (full state reset)
- [x] Debug/admin tools (G=gold, L=lives, K=kill all, N=skip wave, F1=overlay)

---

## Towers & Combat (Phase 2)

### Tower Types
- [x] Lightning tower (fast single-target, magic damage)
- [x] Earth tower (golem blocking system)
- [x] Ice tower (slow debuff 50% for 2s, magic damage)
- [x] Fire tower (AoE splash + burn DOT 3dps/3s)
- [x] Tower upgrades (3 levels per tower, increasing stats)
- [x] Tower sell mechanic (60% refund)
- [x] Tower range indicator (thin ring on select)
- [x] Tower models swapped to match old game (lightning↔fire)
- [ ] Tower mesh part coloring per element (OLD — towers had named sub-meshes colored per element)
- [x] Tower specializations (8 specs: Storm Spire, Railgun, Mountain King, Bramble Grove, Blizzard Tower, Shatter Mage, Inferno Cannon, Meteor Tower)
- [x] Tower targeting priority — targets first enemy on path (furthest along)
- [x] Muzzle flash on tower fire
- [ ] Per-element tower idle + recoil animations (OLD)
- [x] Tower placement animation — easeOutBack scale bounce
- [x] Upgrade flash effect — emissive spike on upgrade
- [x] Upgrade indicator (visual indication of upgraded towers)

### Combat System
- [x] Damage types (physical vs magic)
- [x] Armor / magic resistance reduction
- [x] Burn debuff (fire)
- [x] Slow debuff (ice)
- [ ] Stun debuff (OLD — certain specs)
- [x] Death effects (expanding sphere + fade)
- [x] Gold popups on kill (floating gold sphere)
- [x] Damage numbers — floating orbs showing damage dealt (color-coded by severity)
- [x] Enemy tint system (shared models get distinct colors)

### Elemental Synergies
- [x] Ice + Lightning: slowed enemies take +50% bonus true damage from lightning
- [x] Ice + Fire: slowed enemies take 40% fire damage as bonus true damage (thermal shock)
- [x] Earth + Ice: golems near ice towers apply slow to blocked enemies
- [x] Fire + Earth: golems near fire towers apply burn to blocked enemies
- [x] Synergy radius: 8 units from golem to allied tower

---

## Earth Tower / Golems (Phase 2)

- [x] Golems spawn at nearest path point when earth tower built
- [x] Golems physically block ground enemies
- [x] Golem melee attack (damage blocked enemy)
- [x] Blocked enemies attack golem back
- [x] Golem death + automatic respawn (tower still exists)
- [x] Rally point system (set via UI button + click map)
- [x] Golems walk to rally point, face blocked enemies at rally
- [x] Golem animations (idle, walk, attack) with stop_all fix
- [x] Golem material fix (override near-black base_color)
- [x] Golems stand side-by-side (offset rally points)
- [x] Flying enemies bypass golems
- [x] Blocked enemies spread out (don't stack)
- [x] Blocked enemies can't pass through golems
- [x] Unblocked enemies scatter naturally along path when golem dies/sold
- [x] Cleanup orphan golems when tower sold
- [x] Golem respawn timer — 12s delay before respawning
- [x] Golems respawn at player-set rally point (TowerRallyPoint persists across deaths)
- [ ] Upgrading barracks revives dead golems (OLD)
- [ ] 3 golems per tower instead of 2 (OLD had 3)

---

## Enemy System (Phase 2-3)

### Enemy Types — Primordial Era (Level 1)
- [x] Amoeba (basic ground, PinkBlob model)
- [x] Jellyfish (flying, Hywirl model)
- [x] Sporebloom (healer, GreenBlob model, purple tint)
- [x] Trilobite (armored ground, GreenBlob model, larger)
- [x] Sea Scorpion (fast ground, GreenSpikyBlob model)
- [x] Armored Nautilus (heavy armor, GreenSpikyBlob model, blue tint)
- [x] Giant Worm (boss, GreenSpikyBlob model, red-brown tint) — used as Level 2 boss

### Enemy Types — Other Eras (OLD — 44 more enemy types)
- [ ] Prehistoric era: Raptor, Stegosaurus, Parasaurolophus, Triceratops, Compy Healer, Pterodactyl, T-Rex (boss)
- [ ] Stone Age era: Caveman, Sabertooth, Giant Sloth, Mammoth, Giant Eagle, Shaman, Woolly Rhino (boss)
- [ ] Ancient era: Legionary, Chariot, War Elephant, Eagle Scout, Medicus, Siege Tower (boss)
- [ ] Medieval era: Footman, Cavalry, Knight, Wyvern, Priest, Dragon (boss)
- [ ] Industrial era: Clockwork, Steam Bike, Steam Mech, Zeppelin, Engineer, War Train (boss)
- [ ] Modern era: Soldier, Jeep, Tank, Helicopter, Medic, Heavy Tank (boss)
- [ ] Future era: Robot, Drone Swarm, Mech Walker, Hover Drone, Nano Healer, AI Overlord (boss)

### Enemy Behaviors
- [x] Ground movement along waypoints
- [x] Flying enemies (bypass path, elevated Y)
- [x] Healer aura (heal nearby allies)
- [x] Enemy skeletal animations (walk loop, attack when blocked, death animation with delayed despawn)
- [x] Enemy HP bars (OLD — existed in TS, not yet in native)
- [ ] Boss enemies with special mechanics (OLD)

---

## Wave System (Phase 3)

- [x] 10 waves per level with enemy group definitions
- [x] Wave spawning with intervals and delays
- [x] Wave phase tracking (Idle → Spawning → Active)
- [x] "Call Early" mechanic (Space key during active wave for bonus gold)
- [x] Wave start via Space key
- [x] Wave HUD shows current wave / total + call early prompt
- [x] Pulse-based waves — 2-4 pulses per wave with breathing room (OLD — currently single pulse)
- [x] Wave HP/speed scaling per wave number (10% HP + 1.5% speed per wave)
- [ ] Wave preview — show enemy cards with "NEW" badge before each wave (OLD)
- [x] Star rating system — ≥90% lives = 3 stars, ≥50% = 2, ≥1% = 1 (OLD)

---

## Tower Specializations (Phase 3)

- [x] 2 specialization branches per tower at max level (OLD)
- [x] Storm Spire (Lightning A) — chain lightning jumps between enemies
- [x] Railgun Tower (Lightning B) — massive single-target sniper bolt
- [x] Mountain King (Earth A) — fewer but tankier golems, AoE slam
- [x] Bramble Grove (Earth B) — vine roots that slow + damage
- [x] Blizzard Tower (Ice A) — constant AoE slow field
- [x] Shatter Mage (Ice B) — frozen enemies take 3x crit damage
- [x] Inferno Cannon (Fire A) — napalm zones, burning ground DOT
- [x] Meteor Tower (Fire B) — long-range targeted meteor strikes
- [x] Specialization choice UI at max tower level
- [ ] Storm Spire lightning bolt visuals (jagged line effect)

---

## Hero System (Phase 4)

- [x] Hero entity — tap to move, auto-attack nearby enemies
- [ ] 5 heroes: Aethon (balanced), Voltra (fast DPS), Gorath (tank), Cryo (support), Ignis (AoE)
- [ ] 3 abilities per hero with cooldowns
- [x] Hero HUD (HP bar, ability buttons with cooldown, respawn countdown)
- [x] Hero death + respawn timer (12s)
- [x] 3 abilities per hero with cooldowns and UI buttons
- [x] Hero heals to full on wave start
- [x] Hero passive HP regeneration (5% max HP/s after 3s without damage)
- [x] Hero respawns at death location (not fixed spawn point)
- [ ] Hero select on level select screen
- [ ] Hero procedural visuals (colored capsules + unique geometry)

### Global Abilities
- [ ] Meteor Strike — 80 damage in 2.5 radius, 5 dps burn 3s, 60s cooldown
- [ ] Elemental Reinforcements — summon 2 temp golems anywhere, 30s cooldown

---

## Levels & Content (Phase 5-6)

### Levels
- [x] Level 1 — Primordial Pools (10 waves, 6 enemy types)
- [x] Level 2 — Primordial Depths (10 waves, GiantWorm boss, different path layout)
- [ ] Levels 3-4 — Prehistoric (Jurassic Jungle + second)
- [ ] Levels 5-6 — Stone Age
- [ ] Levels 7-8 — Ancient
- [ ] Levels 9-10 — Medieval
- [ ] Levels 11-12 — Industrial
- [ ] Levels 13-14 — Modern
- [ ] Levels 15-16 — Future
- [ ] Level 17 — Atlantis (bonus, unlocks on Medium completion)

### Themes (per-era visual settings)
- [x] Primordial theme (dark purple swamp, bioluminescence)
- [ ] Prehistoric theme (jungles, volcanoes, lava)
- [ ] Stone Age theme (frozen tundra, glaciers)
- [ ] Ancient theme (columns, stone roads)
- [ ] Medieval theme (castles, forests)
- [ ] Industrial theme (factories, smokestacks)
- [ ] Modern theme (urban, military)
- [ ] Future theme (neon, holograms)
- [ ] Atlantis theme (deep ocean blue/teal)

### Environment
- [x] Scenery scattering (rocks, ferns, palm trees avoiding path + build spots)
- [ ] Per-theme scenery models
- [ ] Lava streams for Prehistoric levels (animated shader material)

---

## UI & Menus (Phase 5-6)

### In-Game UI
- [x] HUD — gold, lives, wave counter
- [x] Build menu — 4 tower type buttons with costs
- [x] Tower panel — stats, upgrade, sell, rally point
- [x] World-positioned UI (panels appear near clicked spot)
- [x] Game Over screen with Play Again
- [x] Enemy HP bars above enemies (OLD)
- [x] Damage numbers floating text (OLD)
- [x] Speed toggle (1x/2x/3x) (OLD)
- [x] Pause button + pause overlay (OLD)
- [ ] Mute button (OLD)
- [ ] Tower radial build menu instead of list (OLD — appeared as radial on mobile)

### Menu Screens
- [ ] Main menu (Campaign, Upgrades, Tutorial, Logbook, Settings)
- [ ] Level select — Kingdom Rush-style node-path map with era regions
- [ ] Hero select picker
- [ ] Settings menu (SFX volume, music volume sliders)
- [ ] Tutorial — 7-step guided walkthrough for first level
- [ ] Logbook / Bestiary (enemies by era + tower info)
- [ ] Loading screen with splash image + progress bar

---

## Audio (Phase 4-5)

- [x] Audio asset loading with all_loaded guard
- [x] Battle music (looping)
- [x] Wave start/complete SFX
- [x] Tower attack SFX
- [x] Enemy death SFX
- [x] Tower build/sell/upgrade SFX
- [ ] Per-element tower attack SFX (currently all play same sound) (OLD)
- [ ] Boss battle music (switch on last 2 waves) (OLD)
- [ ] Menu music (OLD)
- [ ] Victory/defeat fanfares (OLD)
- [ ] Music crossfade transitions (OLD)
- [ ] Volume sliders (SFX + music separate) (OLD)
- [ ] Enemy leak SFX (OLD)

---

## Visual Effects (Phase 4)

- [x] Death burst effect (expanding sphere + fade)
- [x] Gold popup effect (rising sphere + fade)
- [x] Muzzle flash on tower fire
- [x] Range indicator (thin ring on tower select)
- [x] Projectile trails (stretch along velocity)
- [ ] Lightning bolt visual for Storm Spire (OLD — jagged line geometry)
- [ ] Meteor impact effect (OLD — falling meteor + ring + scorch)
- [x] Camera shake (on enemy leak)
- [x] Tower placement bounce animation (OLD)
- [x] Upgrade flash effect (OLD)
- [x] Bloom post-processing (Bevy built-in, intensity 0.15)

---

## Meta-Progression (Phase 5)

- [ ] Save/load system (persistent storage)
- [ ] Star tracking per level per difficulty
- [ ] 5 meta-upgrades purchasable with stars:
  - [ ] War Chest — bonus starting gold
  - [ ] Elemental Fury — bonus damage
  - [ ] Far Sight — bonus tower range
  - [ ] Salvage Expert — better sell refund
  - [ ] Tactical Mastery — shorter ability cooldowns
- [ ] Upgrade shop UI on level select
- [ ] Difficulty modes (Easy/Medium/Hard multipliers)
- [ ] Level locking (must beat previous level)

---

## Android & Mobile (Phase 5)

- [x] Cargo.toml lib + bin setup (cdylib for Android)
- [x] lib.rs with #[bevy_main] entry point
- [x] Android Gradle project (manifest, MainActivity, build.gradle)
- [x] cargo-ndk installed
- [x] aarch64-linux-android Rust target installed
- [ ] NDK installed (in progress — 30.0.14904198)
- [ ] First successful Android build
- [ ] Deploy to device and test
- [ ] Touch input (tap to build, tap to move hero, pinch to zoom)
- [ ] Screen orientation lock (landscape)
- [ ] Safe area handling (notch/nav bar)
- [ ] Fullscreen immersive mode
- [ ] Back button handling (pause)
- [ ] App lifecycle (auto-pause on background)
- [ ] Performance profiling on device
- [ ] Disable shadow maps on Android (known Bevy segfault)
- [ ] Disable MSAA on Android (known Bevy panic)

---

## Monetization (Phase 7 — Future)

- [ ] App-open interstitial ad
- [ ] Revive via rewarded ad on defeat
- [ ] Remove Ads IAP ($1.99)
- [ ] Gold Multiplier 2x IAP ($3.99)
- [ ] Store UI
- [ ] AdMob integration
- [ ] RevenueCat integration
- [ ] GDPR consent dialog

---

## Bonus Content (Post-Launch)

- [ ] Atlantis bonus world (unlocks on Medium completion)
- [ ] Fantasy world (orcs, trolls, dark wizards)
- [ ] Mythology world (minotaurs, hydras, cyclops)
- [ ] Underworld world (demons, skeletons, ghosts)
- [ ] Space world (aliens, UFOs)
- [ ] Pirate world (pirates, cannon ships)
- [ ] Insectoid world (giant ants, swarmers)
- [ ] Steampunk world (clockwork soldiers, airships)

---

## Priority Order

1. **Android build working** — validate no GC pauses (the whole reason for the port)
2. **Enemy HP bars** — critical gameplay feedback
3. **Pulse-based waves** — matches old game's fight-breathe-fight rhythm
4. **Hero system** — core differentiator from generic TD
5. **Tower specializations** — endgame depth
6. **More levels** (at least 4-6 for content)
7. **Main menu + level select** — proper game flow
8. **Save/load + meta-progression** — retention loop
9. **All 51 enemy types** — full content
10. **Polish** (animations, effects, camera shake, damage numbers)
11. **Monetization** — revenue
12. **Bonus worlds** — post-launch content
