use bevy::prelude::*;
use crate::components::*;
use crate::data::*;
use crate::resources::*;
use crate::states::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Playing),
            setup_hud.run_if(|needs: Res<crate::resources::NeedsFreshSetup>| needs.0),
        );
        app.add_systems(
            Update,
            (
                update_hud,
                update_wave_button,
                handle_wave_button,
                manage_panels,
                handle_build_buttons,
                handle_tower_buttons,
                handle_rally_point_button,
                handle_rally_point_click,
                update_button_affordability,
            )
                .run_if(in_state(AppState::Playing)),
        );

        // Hero HUD
        app.add_systems(
            OnEnter(AppState::Playing),
            setup_hero_hud.run_if(|needs: Res<crate::resources::NeedsFreshSetup>| needs.0),
        );
        app.add_systems(
            Update,
            (
                update_hero_hud,
                handle_hero_hud_click,
                handle_ability_buttons,
                update_ability_cooldowns,
                handle_spec_buttons,
            ).run_if(in_state(AppState::Playing)),
        );

        // Pause and speed buttons work during Playing state
        app.add_systems(
            Update,
            (
                handle_pause_button,
                handle_speed_button,
            ).run_if(in_state(AppState::Playing)),
        );

        // Pause screen
        app.init_resource::<PendingConfirm>();
        app.add_systems(OnEnter(AppState::Paused), setup_pause_screen);
        app.add_systems(
            Update,
            (handle_pause_buttons, handle_confirm_dialog).run_if(in_state(AppState::Paused)),
        );
        app.add_systems(OnExit(AppState::Paused), cleanup_pause_screen);

        // Game over screen
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        app.add_systems(
            Update,
            handle_restart_button.run_if(in_state(AppState::GameOver)),
        );
        app.add_systems(OnExit(AppState::GameOver), cleanup_game_over_screen);

        // Main menu
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu);
        app.add_systems(Update, handle_main_menu.run_if(in_state(AppState::MainMenu)));
        app.add_systems(OnExit(AppState::MainMenu), cleanup_menu_screen);

        // Level select
        app.add_systems(OnEnter(AppState::LevelSelect), setup_level_select);
        app.add_systems(Update, (handle_level_select, handle_admin_panel).run_if(in_state(AppState::LevelSelect)));
        app.add_systems(OnExit(AppState::LevelSelect), (cleanup_menu_screen, cleanup_admin_panel));

        // Hero select
        app.add_systems(OnEnter(AppState::HeroSelect), setup_hero_select);
        app.add_systems(Update, (handle_hero_select, handle_admin_panel).run_if(in_state(AppState::HeroSelect)));
        app.add_systems(OnExit(AppState::HeroSelect), (cleanup_menu_screen, cleanup_hero_preview, cleanup_admin_panel));

        // Upgrade Shop
        app.add_systems(OnEnter(AppState::UpgradeShop), setup_upgrade_shop);
        app.add_systems(Update, handle_upgrade_shop.run_if(in_state(AppState::UpgradeShop)));
        app.add_systems(OnExit(AppState::UpgradeShop), cleanup_menu_screen);

        // Logbook
        app.add_systems(OnEnter(AppState::Logbook), setup_logbook);
        app.add_systems(Update, handle_logbook.run_if(in_state(AppState::Logbook)));
        app.add_systems(OnExit(AppState::Logbook), cleanup_menu_screen);

        // Credits
        app.add_systems(OnEnter(AppState::Credits), setup_credits);
        app.add_systems(Update, handle_credits.run_if(in_state(AppState::Credits)));
        app.add_systems(OnExit(AppState::Credits), cleanup_menu_screen);
    }
}

// ---------------------------------------------------------------------------
// Marker components for UI elements
// ---------------------------------------------------------------------------

#[derive(Component)]
struct HudRoot;
#[derive(Component)]
struct GoldText;
#[derive(Component)]
struct LivesText;
#[derive(Component)]
struct WaveText;
#[derive(Component)]
struct BuildMenuRoot;
#[derive(Component)]
struct TowerPanelRoot;
#[derive(Component)]
struct BuildTowerButton(Element);
#[derive(Component)]
struct UpgradeButton {
    cost: u32,
}
#[derive(Component)]
struct SellButton;
#[derive(Component)]
struct SpecButton {
    spec: crate::data::TowerSpecialization,
    cost: u32,
}
#[derive(Component)]
struct RallyPointButton;
#[derive(Component)]
struct TowerInfoText;
#[derive(Component)]
struct RallyPointPrompt;
#[derive(Component)]
struct WaveButton;
#[derive(Component)]
struct WaveButtonText;
#[derive(Component)]
struct SpeedButton;
#[derive(Component)]
struct SpeedButtonText;
#[derive(Component)]
struct PauseButton;
#[derive(Component)]
struct PauseScreenRoot;
#[derive(Component)]
struct ResumeButton;
#[derive(Component)]
struct PauseRestartButton;
#[derive(Component)]
struct PauseQuitButton;
#[derive(Component)]
struct ConfirmDialog;
#[derive(Component)]
struct ConfirmYesButton;
#[derive(Component)]
struct ConfirmNoButton;
/// What action the confirm dialog is for.
#[derive(Resource, Default, PartialEq, Clone, Copy)]
enum PendingConfirm {
    #[default]
    None,
    Restart,
    Quit,
}
#[derive(Component)]
struct MenuScreenRoot;
#[derive(Component)]
struct MenuCamera;
#[derive(Component)]
struct MenuButton(MenuAction);
#[derive(Clone, Copy)]
enum MenuAction {
    Campaign,
    Shop,
    Logbook,
    ModelDebug,
    SelectLevel(u32),
    SelectHero(crate::data::HeroType),
    BackToMenu,
    BackToLevelSelect,
    StartGame,
    LogbookEnemies,
    LogbookTowers,
    LogbookBack,
    BuyUpgrade(crate::data::UpgradeKind),
    Credits,
}
#[derive(Component)]
struct LogbookPageRoot;
#[derive(Component)]
struct HeroPreviewRoot;
#[derive(Component)]
struct HeroPreviewModel;
#[derive(Component)]
struct AdminPanelRoot;
#[derive(Component)]
struct AdminUnlockLevelsButton;
#[derive(Component)]
struct AdminUnlockHeroesButton;

// ---------------------------------------------------------------------------
// HUD setup & update
// ---------------------------------------------------------------------------

fn setup_hud(mut commands: Commands, old_huds: Query<Entity, With<HudRoot>>) {
    // Clean up old HUD if restarting
    for entity in &old_huds {
        commands.entity(entity).despawn_recursive();
    }

    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                column_gap: Val::Px(32.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Gold: 220"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
                GoldText,
            ));
            parent.spawn((
                Text::new("Lives: 20"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                LivesText,
            ));
            parent.spawn((
                Text::new("Wave: 0/10"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::WHITE),
                WaveText,
            ));
            // Send Wave / Call Early button
            parent
                .spawn((
                    Button,
                    WaveButton,
                    Node {
                        height: Val::Px(44.0),
                        padding: UiRect::horizontal(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.35, 0.2, 0.9)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Send Wave"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                        WaveButtonText,
                    ));
                });
            // Speed button (1x / 2x / 3x)
            parent
                .spawn((
                    Button,
                    SpeedButton,
                    Node {
                        height: Val::Px(44.0),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.4, 0.7)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("1x"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                        SpeedButtonText,
                    ));
                });
            // Pause button
            parent
                .spawn((
                    Button,
                    PauseButton,
                    Node {
                        height: Val::Px(44.0),
                        width: Val::Px(44.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("||"),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            // Debug toggle button
            parent
                .spawn((
                    Button,
                    crate::systems::debug::DebugToggleButton,
                    Node {
                        height: Val::Px(44.0),
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.3, 0.0, 0.5)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("DBG"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    ));
                });
        });
}

fn update_hud(
    game: Res<GameData>,
    wave: Res<WaveState>,
    mut gold_q: Query<&mut Text, (With<GoldText>, Without<LivesText>, Without<WaveText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<GoldText>, Without<WaveText>)>,
    mut wave_q: Query<&mut Text, (With<WaveText>, Without<GoldText>, Without<LivesText>)>,
) {
    if let Ok(mut t) = gold_q.get_single_mut() {
        t.0 = format!("Gold: {}", game.gold);
    }
    if let Ok(mut t) = lives_q.get_single_mut() {
        t.0 = format!("Lives: {}", game.lives);
    }
    if let Ok(mut t) = wave_q.get_single_mut() {
        let status = match wave.phase {
            WavePhase::Idle if game.wave_number >= game.max_waves => "[Complete!]",
            WavePhase::Idle => "[Ready]",
            WavePhase::Spawning => "[Spawning...]",
            WavePhase::PulsePause => "[Next pulse...]",
            WavePhase::Active if game.wave_number < game.max_waves => "[Call Early]",
            WavePhase::Active => "[Active]",
        };
        t.0 = format!("Wave: {}/{}  {}", game.wave_number, game.max_waves, status);
    }
}

/// Updates wave button text and color based on current wave state.
fn update_wave_button(
    game: Res<GameData>,
    wave: Res<WaveState>,
    mut btn_q: Query<&mut BackgroundColor, With<WaveButton>>,
    mut text_q: Query<&mut Text, With<WaveButtonText>>,
) {
    let Ok(mut bg) = btn_q.get_single_mut() else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };

    match wave.phase {
        WavePhase::Idle if game.wave_number >= game.max_waves => {
            text.0 = "Complete!".into();
            bg.0 = Color::srgba(0.3, 0.3, 0.3, 0.5);
        }
        WavePhase::Idle => {
            text.0 = "Send Wave".into();
            bg.0 = Color::srgba(0.2, 0.5, 0.2, 0.9);
        }
        WavePhase::Active if game.wave_number < game.max_waves => {
            text.0 = "Call Early".into();
            bg.0 = Color::srgba(0.5, 0.35, 0.1, 0.9);
        }
        _ => {
            text.0 = "In Progress...".into();
            bg.0 = Color::srgba(0.3, 0.3, 0.3, 0.5);
        }
    }
}

/// Handles wave button press — sets the WaveButtonPressed resource.
fn handle_wave_button(
    interactions: Query<&Interaction, (Changed<Interaction>, With<WaveButton>)>,
    mut wave_btn: ResMut<WaveButtonPressed>,
    wave: Res<WaveState>,
    game: Res<GameData>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Only trigger in states where Space would work
        let can_start = matches!(wave.phase, WavePhase::Idle) && game.wave_number < game.max_waves;
        let can_call_early = matches!(wave.phase, WavePhase::Active) && game.wave_number < game.max_waves;
        if can_start || can_call_early {
            wave_btn.0 = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Panel management — spawn/despawn build menu & tower panel based on Selection
// ---------------------------------------------------------------------------

fn manage_panels(
    mut commands: Commands,
    selection: Res<Selection>,
    build_menus: Query<Entity, With<BuildMenuRoot>>,
    tower_panels: Query<Entity, With<TowerPanelRoot>>,
    prompts: Query<Entity, With<RallyPointPrompt>>,
    towers: Query<(&Element, &TowerLevel, &TowerInvestment, &AttackDamage, &AttackRange, &Transform, Option<&TowerSpec>)>,
    spots: Query<&Transform, With<BuildSpot>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    game: Res<GameData>,
) {
    if !selection.is_changed() {
        return;
    }

    // Despawn old panels and prompts
    for entity in &build_menus {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &tower_panels {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &prompts {
        commands.entity(entity).despawn_recursive();
    }

    // Get camera and window for world-to-screen projection
    let Ok((camera, cam_transform)) = camera_query.get_single() else { return };
    let Ok(window) = windows.get_single() else { return };
    let window_width = window.width();
    let window_height = window.height();

    match *selection {
        Selection::None => {}
        Selection::Hero => {} // Hero HUD handles this
        Selection::SettingRallyPoint(_) => {} // Prompt shown separately
        Selection::BuildSpot(spot_entity) => {
            if let Ok(spot_transform) = spots.get(spot_entity) {
                let screen_pos = world_to_screen(camera, cam_transform, spot_transform.translation, window_width, window_height);
                spawn_build_menu(&mut commands, &game, screen_pos);
            }
        }
        Selection::Tower(tower_entity) => {
            if let Ok((element, level, investment, damage, range, tower_transform, spec)) =
                towers.get(tower_entity)
            {
                let screen_pos = world_to_screen(camera, cam_transform, tower_transform.translation, window_width, window_height);
                spawn_tower_panel(
                    &mut commands,
                    *element,
                    level.0,
                    investment.0,
                    damage.0,
                    range.0,
                    &game,
                    screen_pos,
                    spec.map(|s| s.0),
                );
            }
        }
    }
}

/// Project a world position to screen-space (left, top) for UI positioning.
/// Offsets the panel to the right of the object so it doesn't cover it.
fn world_to_screen(
    camera: &Camera,
    cam_transform: &GlobalTransform,
    world_pos: Vec3,
    window_width: f32,
    window_height: f32,
) -> (f32, f32) {
    let Some(ndc) = camera.world_to_ndc(cam_transform, world_pos) else {
        return (window_width - 220.0, 80.0); // fallback
    };
    // NDC is [-1, 1], convert to screen pixels
    let x = (ndc.x + 1.0) * 0.5 * window_width + 60.0; // offset right of object
    let y = (1.0 - ndc.y) * 0.5 * window_height - 40.0; // slightly above

    // Clamp so panel stays on screen
    let panel_width = 210.0;
    let panel_height = 250.0;
    let left = x.clamp(0.0, (window_width - panel_width).max(0.0));
    let top = y.clamp(0.0, (window_height - panel_height).max(0.0));
    (left, top)
}

// ---------------------------------------------------------------------------
// Build menu — 4 tower type buttons
// ---------------------------------------------------------------------------

fn spawn_build_menu(commands: &mut Commands, game: &GameData, screen_pos: (f32, f32)) {
    let tower_options = [
        (Element::Lightning, "Lightning", Color::srgb(1.0, 0.93, 0.27)),
        (Element::Earth, "Earth", Color::srgb(0.53, 0.67, 0.27)),
        (Element::Ice, "Ice", Color::srgb(0.27, 0.8, 1.0)),
        (Element::Fire, "Fire", Color::srgb(1.0, 0.4, 0.13)),
    ];

    commands
        .spawn((
            BuildMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.0),
                top: Val::Px(screen_pos.1),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.05, 0.15, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Build Tower"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::WHITE),
            ));

            for (element, name, color) in tower_options {
                let cost = tower_base_cost(element);
                let affordable = game.gold >= cost;
                let bg = if affordable {
                    Color::srgba(0.2, 0.15, 0.25, 0.9)
                } else {
                    Color::srgba(0.15, 0.1, 0.15, 0.5)
                };

                parent
                    .spawn((
                        Button,
                        BuildTowerButton(element),
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("{} ({}g)", name, cost)),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(if affordable { color } else { Color::srgb(0.4, 0.4, 0.4) }),
                        ));
                    });
            }
        });
}

// ---------------------------------------------------------------------------
// Tower panel — info, upgrade, sell
// ---------------------------------------------------------------------------

fn spawn_tower_panel(
    commands: &mut Commands,
    element: Element,
    level: u8,
    investment: u32,
    damage: f32,
    range: f32,
    game: &GameData,
    screen_pos: (f32, f32),
    spec: Option<crate::data::TowerSpecialization>,
) {
    let stats = tower_stats(element, level);
    let has_spec = spec.is_some();
    let can_upgrade = level < 2;
    let upgrade_cost = if can_upgrade {
        tower_stats(element, level + 1).cost
    } else {
        0
    };
    let sell_value = (investment as f32 * SELL_REFUND_RATE) as u32;
    let color = element_color(element);

    commands
        .spawn((
            TowerPanelRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.0),
                top: Val::Px(screen_pos.1),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                min_width: Val::Px(200.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.05, 0.15, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // Tower name — show specialization name if specialized
            let title = if let Some(s) = spec {
                let specs = crate::data::element_specializations(element);
                specs.iter()
                    .find(|(st, _)| *st == s)
                    .map(|(_, d)| d.name.to_string())
                    .unwrap_or_else(|| format!("{} (Lv {})", stats.name, level + 1))
            } else {
                format!("{} (Lv {})", stats.name, level + 1)
            };
            parent.spawn((
                Text::new(title),
                TextFont { font_size: 20.0, ..default() },
                TextColor(color),
            ));

            // Stats
            parent.spawn((
                Text::new(format!("DMG: {:.0}  RNG: {:.1}", damage, range)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                TowerInfoText,
            ));

            // Upgrade button
            if can_upgrade {
                let affordable = game.gold >= upgrade_cost;
                parent
                    .spawn((
                        Button,
                        UpgradeButton { cost: upgrade_cost },
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if affordable {
                            Color::srgba(0.2, 0.4, 0.2, 0.9)
                        } else {
                            Color::srgba(0.15, 0.15, 0.15, 0.5)
                        }),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("Upgrade ({}g)", upgrade_cost)),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(if affordable {
                                Color::WHITE
                            } else {
                                Color::srgb(0.4, 0.4, 0.4)
                            }),
                        ));
                    });
            } else if !has_spec {
                // Show specialization choices
                parent.spawn((
                    Text::new("Specialize:"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.0)),
                ));
                let specs = crate::data::element_specializations(element);
                for (spec_type, spec_def) in &specs {
                    let affordable = game.gold >= spec_def.cost;
                    parent
                        .spawn((
                            Button,
                            SpecButton { spec: *spec_type, cost: spec_def.cost },
                            Node {
                                width: Val::Px(200.0),
                                min_height: Val::Px(40.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(if affordable {
                                Color::srgba(0.3, 0.2, 0.4, 0.9)
                            } else {
                                Color::srgba(0.15, 0.15, 0.15, 0.5)
                            }),
                            BorderRadius::all(Val::Px(6.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(format!("{} ({}g)", spec_def.name, spec_def.cost)),
                                TextFont { font_size: 14.0, ..default() },
                                TextColor(if affordable { Color::WHITE } else { Color::srgb(0.4, 0.4, 0.4) }),
                            ));
                            btn.spawn((
                                Text::new(spec_def.description),
                                TextFont { font_size: 11.0, ..default() },
                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            ));
                        });
                }
            } else {
                parent.spawn((
                    Text::new("SPECIALIZED"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.0)),
                ));
            }

            // Sell button
            parent
                .spawn((
                    Button,
                    SellButton,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.4, 0.15, 0.15, 0.9)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("Sell ({}g)", sell_value)),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.6, 0.6)),
                    ));
                });

            // Rally Point button for Earth towers
            if element == Element::Earth {
                parent
                    .spawn((
                        Button,
                        RallyPointButton,
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.9)),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Set Rally Point"),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::srgb(0.53, 0.67, 0.27)),
                        ));
                    });
            }
        });
}

// ---------------------------------------------------------------------------
// Button handlers
// ---------------------------------------------------------------------------

/// When a build menu button is clicked, place that tower type on the selected spot.
fn handle_build_buttons(
    mut commands: Commands,
    interactions: Query<(&Interaction, &BuildTowerButton), Changed<Interaction>>,
    mut selection: ResMut<Selection>,
    mut game: ResMut<GameData>,
    mut spots: Query<&mut BuildSpot>,
    asset_server: Res<AssetServer>,
    audio_assets: Option<Res<crate::systems::audio::AudioAssets>>,
    save_data: Option<Res<crate::save::SaveData>>,
) {
    for (interaction, build_btn) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Selection::BuildSpot(spot_entity) = *selection else {
            continue;
        };

        let element = build_btn.0;
        let stats = tower_stats(element, 0);

        if game.gold < stats.cost {
            continue;
        }

        // Deduct gold
        game.gold -= stats.cost;

        // Mark spot as occupied
        if let Ok(mut spot) = spots.get_mut(spot_entity) {
            spot.occupied = true;
        }

        // Get spot position for tower placement
        // We need the transform, but BuildSpot doesn't have Transform in this query.
        // Use a separate approach: store position from the spot entity's transform.
        let spot_pos = commands.entity(spot_entity).id();
        // Actually, let's just get it from a world query — but we can't add another
        // Query with Transform on BuildSpot due to conflicts. Instead, read it before.
        // Workaround: place tower at a default and fix next frame, OR restructure.
        // Simplest: get GlobalTransform from the entity.

        let scene = asset_server.load(format!("{}#Scene0", stats.model_path));

        commands.spawn((
            SceneRoot(scene),
            // Start at scale 0 — PlacementBounce animates to target
            Transform::from_scale(Vec3::ZERO),
            Tower,
            element,
            TowerLevel(0),
            TowerInvestment(stats.cost),
            BuildSpotRef(spot_pos),
            AttackTimer {
                cooldown: 1.0 / stats.attack_speed,
                elapsed: 0.0,
            },
            AttackRange(stats.range * save_data.as_ref().map(|s| s.tower_range_mult()).unwrap_or(1.0)),
            AttackDamage(stats.damage * save_data.as_ref().map(|s| s.tower_damage_mult()).unwrap_or(1.0)),
            GameWorldEntity,
            PlacementBounce {
                duration: 0.4,
                elapsed: 0.0,
                target_scale: stats.model_scale,
            },
        ));

        // Play build SFX
        if let Some(ref audio) = audio_assets {
            if audio.all_loaded {
                commands.spawn((
                    AudioPlayer(audio.tower_build.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }

        info!("Built {} tower", element);

        *selection = Selection::None;
    }
}

/// Handle upgrade and sell button clicks on the tower panel.
fn handle_tower_buttons(
    mut commands: Commands,
    upgrade_q: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
    sell_q: Query<&Interaction, (Changed<Interaction>, With<SellButton>)>,
    mut selection: ResMut<Selection>,
    mut game: ResMut<GameData>,
    mut towers: Query<(
        &Element,
        &mut TowerLevel,
        &mut TowerInvestment,
        &mut AttackDamage,
        &mut AttackRange,
        &mut AttackTimer,
        &BuildSpotRef,
    )>,
    mut spots: Query<&mut BuildSpot>,
    audio_assets: Option<Res<crate::systems::audio::AudioAssets>>,
    save_data: Option<Res<crate::save::SaveData>>,
) {
    let Selection::Tower(tower_entity) = *selection else {
        return;
    };

    // --- Upgrade ---
    for interaction in &upgrade_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Ok((element, mut level, mut investment, mut damage, mut range, mut timer, _)) =
            towers.get_mut(tower_entity)
        {
            if level.0 >= 2 {
                continue;
            }
            let new_stats = tower_stats(*element, level.0 + 1);
            if game.gold < new_stats.cost {
                continue;
            }

            game.gold -= new_stats.cost;
            investment.0 += new_stats.cost;
            level.0 += 1;
            damage.0 = new_stats.damage * save_data.as_ref().map(|s| s.tower_damage_mult()).unwrap_or(1.0);
            range.0 = new_stats.range * save_data.as_ref().map(|s| s.tower_range_mult()).unwrap_or(1.0);
            timer.cooldown = 1.0 / new_stats.attack_speed;

            // Trigger upgrade flash
            commands.entity(tower_entity).insert(UpgradeFlash { remaining: 0.3 });
        }

        // Play upgrade SFX
        if let Some(ref audio) = audio_assets {
            if audio.all_loaded {
                commands.spawn((
                    AudioPlayer(audio.tower_upgrade.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }

        // Refresh panel by re-triggering selection
        *selection = Selection::None;
    }

    // --- Sell ---
    for interaction in &sell_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Ok((_, _, investment, _, _, _, spot_ref)) = towers.get(tower_entity) {
            let refund_rate = save_data.as_ref().map(|s| s.sell_refund_rate()).unwrap_or(SELL_REFUND_RATE);
            let refund = (investment.0 as f32 * refund_rate) as u32;
            game.gold += refund;

            // Mark build spot as unoccupied
            if let Ok(mut spot) = spots.get_mut(spot_ref.0) {
                spot.occupied = false;
            }

            commands.entity(tower_entity).despawn_recursive();

            // Play sell SFX
            if let Some(ref audio) = audio_assets {
                if audio.all_loaded {
                    commands.spawn((
                        AudioPlayer(audio.tower_sell.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }
        }

        *selection = Selection::None;
    }
}

/// Dynamically update upgrade/spec button colors based on current gold.
fn update_button_affordability(
    game: Res<GameData>,
    mut upgrade_q: Query<(&UpgradeButton, &mut BackgroundColor, &Children)>,
    mut spec_q: Query<(&SpecButton, &mut BackgroundColor, &Children), Without<UpgradeButton>>,
    mut text_colors: Query<&mut TextColor>,
) {
    if !game.is_changed() {
        return;
    }

    for (btn, mut bg, children) in &mut upgrade_q {
        let affordable = game.gold >= btn.cost;
        bg.0 = if affordable {
            Color::srgba(0.2, 0.4, 0.2, 0.9)
        } else {
            Color::srgba(0.15, 0.15, 0.15, 0.5)
        };
        for child in children.iter() {
            if let Ok(mut tc) = text_colors.get_mut(*child) {
                tc.0 = if affordable { Color::WHITE } else { Color::srgb(0.4, 0.4, 0.4) };
            }
        }
    }

    for (btn, mut bg, children) in &mut spec_q {
        let affordable = game.gold >= btn.cost;
        bg.0 = if affordable {
            Color::srgba(0.3, 0.2, 0.4, 0.9)
        } else {
            Color::srgba(0.15, 0.15, 0.15, 0.5)
        };
        for child in children.iter() {
            if let Ok(mut tc) = text_colors.get_mut(*child) {
                // Only update the first child (name+cost text), not description
                tc.0 = if affordable { Color::WHITE } else { Color::srgb(0.4, 0.4, 0.4) };
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rally point
// ---------------------------------------------------------------------------

/// When "Set Rally Point" is clicked, enter rally point placement mode.
fn handle_rally_point_button(
    mut commands: Commands,
    rally_q: Query<&Interaction, (Changed<Interaction>, With<RallyPointButton>)>,
    mut selection: ResMut<Selection>,
    prompts: Query<Entity, With<RallyPointPrompt>>,
) {
    for interaction in &rally_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Selection::Tower(tower_entity) = *selection else {
            continue;
        };

        *selection = Selection::SettingRallyPoint(tower_entity);

        // Show prompt
        for entity in &prompts {
            commands.entity(entity).despawn_recursive();
        }
        commands.spawn((
            RallyPointPrompt,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(50.0),
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Click on the map to set rally point"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.53, 0.67, 0.27)),
            ));
        });
    }
}

/// When in rally point mode, click the ground to set the rally point.
/// Skips the first frame after entering rally mode to avoid the button tap
/// also registering as the rally point click.
fn handle_rally_point_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut selection: ResMut<Selection>,
    mut golems: Query<(&GolemOwner, &mut GolemRallyPoint)>,
    towers: Query<(&Transform, &AttackRange), With<Tower>>,
    prompts: Query<Entity, With<RallyPointPrompt>>,
    ui_interactions: Query<&Interaction, With<Button>>,
    mut was_setting: Local<bool>,
) {
    let Selection::SettingRallyPoint(tower_entity) = *selection else {
        *was_setting = false;
        return;
    };

    // Skip the first frame after entering rally mode — prevents the "Set Rally Point"
    // button tap from also registering as the ground tap
    if !*was_setting {
        *was_setting = true;
        return;
    }

    // Cancel with Escape
    if keys.just_pressed(KeyCode::Escape) {
        *selection = Selection::Tower(tower_entity);
        for entity in &prompts {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    // Get click/tap position
    let screen_pos = if mouse.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.get_single() else { return };
        window.cursor_position()
    } else if let Some(touch) = touches.iter_just_pressed().next() {
        Some(touch.position())
    } else {
        return;
    };

    let Some(screen_pos) = screen_pos else { return };

    // Skip if clicking UI
    for interaction in &ui_interactions {
        if *interaction != Interaction::None {
            return;
        }
    }

    let Ok((camera, cam_transform)) = camera_query.get_single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, screen_pos) else { return };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let mut world_pos = ray.get_point(distance);
    world_pos.y = 0.0;

    // Clamp rally point to tower's attack range
    if let Ok((tower_tf, range)) = towers.get(tower_entity) {
        let tower_pos = Vec3::new(tower_tf.translation.x, 0.0, tower_tf.translation.z);
        let diff = world_pos - tower_pos;
        let dist = diff.length();
        if dist > range.0 {
            world_pos = tower_pos + diff.normalize() * range.0;
        }
    }

    // Save rally point on the tower so it persists across golem death/respawn
    commands.entity(tower_entity).insert(TowerRallyPoint(world_pos));

    // Update rally points for golems belonging to this tower, offset so they stand side by side
    let mut golem_index = 0u32;
    for (owner, mut rally) in &mut golems {
        if owner.0 == tower_entity {
            let offset = if golem_index == 0 {
                Vec3::new(0.8, 0.0, 0.0)
            } else {
                Vec3::new(-0.8, 0.0, 0.0)
            };
            rally.0 = world_pos + offset;
            golem_index += 1;
        }
    }

    // Clean up and go back to tower selection
    for entity in &prompts {
        commands.entity(entity).despawn_recursive();
    }
    *selection = Selection::Tower(tower_entity);

    info!("Rally point set to ({:.1}, {:.1})", world_pos.x, world_pos.z);
}

// ---------------------------------------------------------------------------
// Pause menu
// ---------------------------------------------------------------------------

fn handle_pause_button(
    interactions: Query<&Interaction, (Changed<Interaction>, With<PauseButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Paused);
        }
    }
}

fn handle_speed_button(
    interactions: Query<&Interaction, (Changed<Interaction>, With<SpeedButton>)>,
    mut speed: ResMut<GameSpeed>,
    mut text_q: Query<&mut Text, With<SpeedButtonText>>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            // Cycle: 1x → 2x → 3x → 1x
            speed.0 = match speed.0 as u32 {
                1 => 2.0,
                2 => 3.0,
                _ => 1.0,
            };
            if let Ok(mut text) = text_q.get_single_mut() {
                text.0 = format!("{}x", speed.0 as u32);
            }
        }
    }
}

fn setup_pause_screen(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    time.pause();
    commands
        .spawn((
            PauseScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: 56.0, ..default() },
                TextColor(Color::WHITE),
            ));

            // Resume button
            parent
                .spawn((
                    Button,
                    ResumeButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.5, 0.2, 0.9)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Resume"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // Restart button
            parent
                .spawn((
                    Button,
                    PauseRestartButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.9)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Restart"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit button (back to level select)
            parent
                .spawn((
                    Button,
                    PauseQuitButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.4, 0.2, 0.1, 0.9)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Quit"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn handle_pause_buttons(
    mut commands: Commands,
    resume_q: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    restart_q: Query<&Interaction, (Changed<Interaction>, With<PauseRestartButton>)>,
    quit_q: Query<&Interaction, (Changed<Interaction>, With<PauseQuitButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingConfirm>,
    existing_dialog: Query<Entity, With<ConfirmDialog>>,
) {
    // Don't process pause buttons if confirm dialog is open
    if !existing_dialog.is_empty() { return; }

    for interaction in &resume_q {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Playing);
        }
    }
    for interaction in &restart_q {
        if *interaction == Interaction::Pressed {
            *pending = PendingConfirm::Restart;
            spawn_confirm_dialog(&mut commands, "Restart this level?");
        }
    }
    for interaction in &quit_q {
        if *interaction == Interaction::Pressed {
            *pending = PendingConfirm::Quit;
            spawn_confirm_dialog(&mut commands, "Quit to level select?");
        }
    }
}

fn cleanup_pause_screen(
    mut commands: Commands,
    query: Query<Entity, With<PauseScreenRoot>>,
    confirm_q: Query<Entity, With<ConfirmDialog>>,
    mut time: ResMut<Time<Virtual>>,
    mut pending: ResMut<PendingConfirm>,
) {
    time.unpause();
    *pending = PendingConfirm::None;
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &confirm_q {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_confirm_dialog(commands: &mut Commands, message: &str) {
    commands
        .spawn((
            ConfirmDialog,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            GlobalZIndex(20),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(20.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(message),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    panel
                        .spawn(Node {
                            column_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Yes button
                            row.spawn((
                                Button,
                                ConfirmYesButton,
                                Node {
                                    width: Val::Px(120.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.9)),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Yes"),
                                    TextFont { font_size: 22.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });

                            // No button
                            row.spawn((
                                Button,
                                ConfirmNoButton,
                                Node {
                                    width: Val::Px(120.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.9)),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("No"),
                                    TextFont { font_size: 22.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                });
        });
}

fn handle_confirm_dialog(
    mut commands: Commands,
    yes_q: Query<&Interaction, (Changed<Interaction>, With<ConfirmYesButton>)>,
    no_q: Query<&Interaction, (Changed<Interaction>, With<ConfirmNoButton>)>,
    dialog_q: Query<Entity, With<ConfirmDialog>>,
    mut pending: ResMut<PendingConfirm>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<GameData>,
    mut wave: ResMut<WaveState>,
    mut selection: ResMut<Selection>,
    mut speed: ResMut<GameSpeed>,
    mut needs_setup: ResMut<crate::resources::NeedsFreshSetup>,
    mut hero_cmd: ResMut<crate::resources::HeroMoveCommand>,
    game_entities: Query<Entity, With<crate::components::GameWorldEntity>>,
    hud_q: Query<Entity, With<HudRoot>>,
    hero_hud_q: Query<Entity, With<HeroHudRoot>>,
) {
    for interaction in &yes_q {
        if *interaction == Interaction::Pressed {
            let action = *pending;
            if action == PendingConfirm::None { continue; }

            // Common reset
            *game = GameData::default();
            *wave = WaveState::default();
            *selection = Selection::None;
            speed.0 = 1.0;
            hero_cmd.0 = None;
            needs_setup.0 = true;

            match action {
                PendingConfirm::Restart => {
                    // Let OnEnter(Playing) handle cleanup + re-setup
                    next_state.set(AppState::WaitingForWindow);
                }
                PendingConfirm::Quit => {
                    // Manually clean up game world + HUD since we're not re-entering Playing
                    for entity in &game_entities {
                        commands.entity(entity).despawn_recursive();
                    }
                    for entity in &hud_q {
                        commands.entity(entity).despawn_recursive();
                    }
                    for entity in &hero_hud_q {
                        commands.entity(entity).despawn_recursive();
                    }
                    next_state.set(AppState::LevelSelect);
                }
                PendingConfirm::None => {}
            }
            *pending = PendingConfirm::None;
        }
    }
    for interaction in &no_q {
        if *interaction == Interaction::Pressed {
            *pending = PendingConfirm::None;
            for entity in &dialog_q {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Game Over screen
// ---------------------------------------------------------------------------

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
struct RestartButton;

fn setup_game_over_screen(mut commands: Commands, outcome: Res<GameOutcome>, game: Res<GameData>) {
    let title = if outcome.victory { "VICTORY!" } else { "DEFEAT" };
    let title_color = if outcome.victory {
        Color::srgb(1.0, 0.85, 0.0)
    } else {
        Color::srgb(1.0, 0.3, 0.3)
    };

    let stars_text = if outcome.victory {
        let filled: String = (0..outcome.stars).map(|_| '\u{2605}').collect(); // filled star
        let empty: String = (0..(3 - outcome.stars)).map(|_| '\u{2606}').collect(); // empty star
        format!("{}{}", filled, empty)
    } else {
        String::new()
    };

    let lives_text = format!("Lives remaining: {}", game.lives);

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont { font_size: 64.0, ..default() },
                TextColor(title_color),
            ));

            if outcome.victory {
                parent.spawn((
                    Text::new(stars_text),
                    TextFont { font_size: 48.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.0)),
                ));
            }

            parent.spawn((
                Text::new(lives_text),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    Button,
                    RestartButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.9)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Play Again"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn handle_restart_button(
    mut commands: Commands,
    interactions: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<GameData>,
    mut wave: ResMut<WaveState>,
    mut selection: ResMut<Selection>,
    mut speed: ResMut<GameSpeed>,
    mut needs_setup: ResMut<crate::resources::NeedsFreshSetup>,
    mut hero_cmd: ResMut<crate::resources::HeroMoveCommand>,
    build_menus: Query<Entity, With<BuildMenuRoot>>,
    tower_panels: Query<Entity, With<TowerPanelRoot>>,
    prompts: Query<Entity, With<RallyPointPrompt>>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            *game = GameData::default();
            *wave = WaveState::default();
            *selection = Selection::None;
            speed.0 = 1.0;
            hero_cmd.0 = None;
            needs_setup.0 = true;

            // Clean up any lingering UI panels
            for e in &build_menus { commands.entity(e).despawn_recursive(); }
            for e in &tower_panels { commands.entity(e).despawn_recursive(); }
            for e in &prompts { commands.entity(e).despawn_recursive(); }

            // Go through WaitingForWindow for clean state transition
            next_state.set(AppState::WaitingForWindow);
        }
    }
}

fn cleanup_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ---------------------------------------------------------------------------
// Hero HUD — health bar + respawn timer at bottom-left
// ---------------------------------------------------------------------------

#[derive(Component)]
struct HeroHudRoot;

#[derive(Component)]
struct HeroHealthBarFill;

#[derive(Component)]
struct HeroStatusText;

#[derive(Component)]
struct AbilityButton(usize);

#[derive(Component)]
struct AbilityCooldownText(usize);


fn setup_hero_hud(
    mut commands: Commands,
    old: Query<Entity, With<HeroHudRoot>>,
    active_hero: Res<ActiveHeroType>,
) {
    for entity in &old {
        commands.entity(entity).despawn_recursive();
    }

    let defs = crate::data::hero_abilities(active_hero.0);
    let stats = crate::data::hero_stats(active_hero.0);

    commands
        .spawn((
            HeroHudRoot,
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.05, 0.15, 0.85)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // Hero name
            parent.spawn((
                Text::new(stats.name),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.9, 0.75, 1.0)),
                HeroStatusText,
            ));

            // Health bar background
            parent
                .spawn((
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.0, 0.0, 0.8)),
                    BorderRadius::all(Val::Px(3.0)),
                ))
                .with_children(|bg| {
                    bg.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 0.8, 0.0)),
                        BorderRadius::all(Val::Px(3.0)),
                        HeroHealthBarFill,
                    ));
                });

            // Ability buttons row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|row| {
                    for (i, def) in defs.iter().enumerate() {
                        let [r, g, b] = def.color;
                        row.spawn((
                            AbilityButton(i),
                            Button,
                            Node {
                                width: Val::Px(56.0),
                                height: Val::Px(44.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(r * 0.4, g * 0.4, b * 0.4, 0.9)),
                            BorderRadius::all(Val::Px(4.0)),
                        ))
                        .with_children(|btn| {
                            // Ability name (truncated)
                            let short_name: String = def.name.chars().take(6).collect();
                            btn.spawn((
                                Text::new(short_name),
                                TextFont { font_size: 10.0, ..default() },
                                TextColor(Color::srgb(r, g, b)),
                            ));
                            // Cooldown text (hidden when ready)
                            btn.spawn((
                                Text::new(""),
                                TextFont { font_size: 12.0, ..default() },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                                AbilityCooldownText(i),
                            ));
                        });
                    }
                });
        });
}

fn handle_hero_hud_click(
    interactions: Query<&Interaction, (Changed<Interaction>, With<HeroHudRoot>)>,
    mut selection: ResMut<Selection>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            *selection = Selection::Hero;
        }
    }
}

fn update_hero_hud(
    hero_q: Query<(&Health, Option<&HeroRespawnTimer>), With<Hero>>,
    mut fill_q: Query<(&mut Node, &mut BackgroundColor), With<HeroHealthBarFill>>,
    mut text_q: Query<&mut Text, With<HeroStatusText>>,
    mut hud_bg_q: Query<&mut BackgroundColor, (With<HeroHudRoot>, Without<HeroHealthBarFill>)>,
    active_hero: Res<ActiveHeroType>,
    selection: Res<Selection>,
) {
    let Ok((health, respawn)) = hero_q.get_single() else {
        return;
    };
    let stats = crate::data::hero_stats(active_hero.0);

    // Update health bar
    if let Ok((mut node, mut bg_color)) = fill_q.get_single_mut() {
        if respawn.is_some() {
            node.width = Val::Percent(0.0);
        } else {
            let pct = (health.current / health.max * 100.0).clamp(0.0, 100.0);
            node.width = Val::Percent(pct);

            // Color: green → yellow → red
            let ratio = health.current / health.max;
            bg_color.0 = if ratio > 0.5 {
                let t = (1.0 - ratio) * 2.0;
                Color::srgb(t, 1.0, 0.0)
            } else {
                let t = ratio * 2.0;
                Color::srgb(1.0, t, 0.0)
            };
        }
    }

    // Update status text
    if let Ok(mut text) = text_q.get_single_mut() {
        if let Some(timer) = respawn {
            text.0 = format!("{} (Respawn: {:.0}s)", stats.name, timer.remaining);
        } else {
            text.0 = format!("{} ({:.0}/{:.0})", stats.name, health.current, health.max);
        }
    }

    // Highlight HUD when hero is selected
    if let Ok(mut bg) = hud_bg_q.get_single_mut() {
        bg.0 = if matches!(*selection, Selection::Hero) {
            Color::srgba(0.2, 0.1, 0.35, 0.95)
        } else {
            Color::srgba(0.1, 0.05, 0.15, 0.85)
        };
    }
}

fn handle_spec_buttons(
    interactions: Query<(&Interaction, &SpecButton), Changed<Interaction>>,
    mut spec_res: ResMut<crate::systems::tower_spec::SpecializationChosen>,
) {
    for (interaction, btn) in &interactions {
        if *interaction == Interaction::Pressed {
            spec_res.0 = Some(btn.spec);
        }
    }
}

fn handle_ability_buttons(
    interactions: Query<(&Interaction, &AbilityButton), Changed<Interaction>>,
    mut ability_res: ResMut<crate::systems::hero_ability::AbilityActivated>,
) {
    for (interaction, btn) in &interactions {
        if *interaction == Interaction::Pressed {
            ability_res.0 = Some(btn.0);
        }
    }
}

fn update_ability_cooldowns(
    hero_q: Query<&HeroAbilities, With<Hero>>,
    mut cd_text_q: Query<(&mut Text, &AbilityCooldownText)>,
    mut btn_q: Query<(&mut BackgroundColor, &AbilityButton)>,
    active_hero: Res<ActiveHeroType>,
) {
    let Ok(abilities) = hero_q.get_single() else { return };
    let defs = crate::data::hero_abilities(active_hero.0);

    // Update cooldown text
    for (mut text, cd_marker) in &mut cd_text_q {
        let idx = cd_marker.0;
        if idx < 3 {
            let cd = abilities.cooldowns[idx];
            if cd > 0.0 {
                text.0 = format!("{:.0}", cd.ceil());
            } else {
                text.0 = String::new();
            }
        }
    }

    // Update button background (dim when on cooldown)
    for (mut bg, btn) in &mut btn_q {
        let idx = btn.0;
        if idx < 3 {
            let [r, g, b] = defs[idx].color;
            let cd = abilities.cooldowns[idx];
            if cd > 0.0 {
                bg.0 = Color::srgba(r * 0.15, g * 0.15, b * 0.15, 0.9);
            } else {
                bg.0 = Color::srgba(r * 0.4, g * 0.4, b * 0.4, 0.9);
            }
        }
    }
}

// ===========================================================================
// Menu Screens — Main Menu, Level Select, Hero Select, Logbook
// ===========================================================================

fn cleanup_menu_screen(
    mut commands: Commands,
    roots: Query<Entity, With<MenuScreenRoot>>,
    cameras: Query<Entity, With<MenuCamera>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &cameras {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_hero_preview(
    mut commands: Commands,
    preview_entities: Query<Entity, With<HeroPreviewRoot>>,
) {
    for entity in &preview_entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_admin_panel(
    mut commands: Commands,
    panels: Query<Entity, With<AdminPanelRoot>>,
) {
    for entity in &panels {
        commands.entity(entity).despawn_recursive();
    }
}

/// Helper: spawn a styled menu button.
fn spawn_menu_button(parent: &mut ChildBuilder, label: &str, action: MenuAction, width: f32, bg: Color, text_color: Color) {
    parent
        .spawn((
            Button,
            MenuButton(action),
            Node {
                width: Val::Px(width),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 20.0, ..default() },
                TextColor(text_color),
            ));
        });
}

// ---------------------------------------------------------------------------
// Main Menu
// ---------------------------------------------------------------------------

fn setup_main_menu(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<MenuCamera>>,
) {
    commands.insert_resource(ClearColor(Color::srgb(0.05, 0.02, 0.1)));

    // Spawn a camera for UI rendering if one doesn't exist
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera2d,
            MenuCamera,
        ));
    }

    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("Ages of Aether"),
                TextFont { font_size: 56.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));
            root.spawn((
                Text::new("Tower Defense"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.6, 0.5, 0.8)),
                Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
            ));

            // Campaign button
            spawn_menu_button(
                root, "Campaign",
                MenuAction::Campaign,
                280.0,
                Color::srgba(0.2, 0.15, 0.4, 0.9),
                Color::WHITE,
            );
            // Shop button
            spawn_menu_button(
                root, "Shop",
                MenuAction::Shop,
                280.0,
                Color::srgba(0.1, 0.2, 0.35, 0.9),
                Color::srgb(0.5, 0.8, 1.0),
            );
            // Logbook button
            spawn_menu_button(
                root, "Logbook",
                MenuAction::Logbook,
                280.0,
                Color::srgba(0.15, 0.25, 0.15, 0.9),
                Color::srgb(0.7, 0.9, 0.7),
            );
            // Credits button
            spawn_menu_button(
                root, "Credits",
                MenuAction::Credits,
                280.0,
                Color::srgba(0.15, 0.15, 0.2, 0.9),
                Color::srgb(0.8, 0.8, 0.9),
            );
            // Model debug button
            spawn_menu_button(
                root, "Model Debug",
                MenuAction::ModelDebug,
                280.0,
                Color::srgba(0.3, 0.15, 0.0, 0.7),
                Color::srgb(1.0, 0.6, 0.3),
            );
        });
}

fn handle_main_menu(
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            MenuAction::Campaign => next_state.set(AppState::LevelSelect),
            MenuAction::Shop => next_state.set(AppState::UpgradeShop),
            MenuAction::Logbook => next_state.set(AppState::Logbook),
            MenuAction::Credits => next_state.set(AppState::Credits),
            MenuAction::ModelDebug => next_state.set(AppState::ModelDebug),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Level Select
// ---------------------------------------------------------------------------

fn setup_level_select(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<MenuCamera>>,
    save: Option<Res<crate::save::SaveData>>,
    admin: Res<crate::resources::AdminUnlocks>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    let save_data = save.map(|s| s.clone()).unwrap_or_default();

    // Admin panel
    spawn_admin_panel(&mut commands, &admin);

    build_level_select_screen(&mut commands, &save_data, &admin);
}

fn build_level_select_screen(
    commands: &mut Commands,
    save_data: &crate::save::SaveData,
    admin: &crate::resources::AdminUnlocks,
) {
    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Header row with title and gem count
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            }).with_children(|header| {
                header.spawn((
                    Text::new("Select Level"),
                    TextFont { font_size: 36.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));
                header.spawn((
                    Text::new(format!("Gems: {}", save_data.aether_gems)),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.8, 1.0)),
                ));
            });

            // Scrollable level cards
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(14.0),
                    row_gap: Val::Px(14.0),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    flex_grow: 1.0,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(|row| {
                for level_num in 1..=crate::data::MAX_LEVELS {
                    let info = crate::data::level_info(level_num);
                    let idx = (level_num as usize).saturating_sub(1);
                    let stars = if idx < save_data.level_stars.len() { save_data.level_stars[idx] } else { 0 };

                    // Level is unlocked if admin override, level 1, or previous level beaten
                    let unlocked = admin.all_levels || level_num == 1 || {
                        let prev_idx = idx.saturating_sub(1);
                        prev_idx < save_data.level_stars.len() && save_data.level_stars[prev_idx] > 0
                    };

                    let bg = if unlocked {
                        Color::srgba(0.15, 0.1, 0.25, 0.9)
                    } else {
                        Color::srgba(0.1, 0.1, 0.1, 0.6)
                    };
                    let text_color = if unlocked { Color::WHITE } else { Color::srgb(0.4, 0.4, 0.4) };

                    row.spawn((
                        Button,
                        MenuButton(MenuAction::SelectLevel(level_num)),
                        Node {
                            width: Val::Px(200.0),
                            min_height: Val::Px(120.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderRadius::all(Val::Px(10.0)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(format!("Level {}", level_num)),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(if unlocked { Color::srgb(0.6, 0.5, 0.8) } else { Color::srgb(0.3, 0.3, 0.4) }),
                        ));
                        card.spawn((
                            Text::new(if unlocked { info.name } else { "LOCKED" }),
                            TextFont { font_size: 20.0, ..default() },
                            TextColor(text_color),
                        ));
                        if unlocked {
                            card.spawn((
                                Text::new(info.era),
                                TextFont { font_size: 12.0, ..default() },
                                TextColor(Color::srgb(0.7, 0.6, 0.3)),
                            ));
                            // Star display
                            if stars > 0 {
                                let filled: String = (0..stars).map(|_| '\u{2605}').collect();
                                let empty: String = (0..(3u8.saturating_sub(stars))).map(|_| '\u{2606}').collect();
                                card.spawn((
                                    Text::new(format!("{}{}", filled, empty)),
                                    TextFont { font_size: 22.0, ..default() },
                                    TextColor(Color::srgb(1.0, 0.85, 0.0)),
                                ));
                            }
                            card.spawn((
                                Text::new(format!("{} waves", info.waves)),
                                TextFont { font_size: 11.0, ..default() },
                                TextColor(Color::srgb(0.5, 0.8, 0.5)),
                            ));
                        }
                    });
                }
            });

            // Back button
            spawn_menu_button(
                root, "Back",
                MenuAction::BackToMenu,
                160.0,
                Color::srgba(0.3, 0.15, 0.15, 0.9),
                Color::srgb(1.0, 0.7, 0.7),
            );
        });
}

fn handle_level_select(
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut current_level: ResMut<crate::resources::CurrentLevel>,
    save: Option<Res<crate::save::SaveData>>,
    admin: Res<crate::resources::AdminUnlocks>,
) {
    let save_data = save.map(|s| s.clone()).unwrap_or_default();
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            MenuAction::SelectLevel(level) => {
                let idx = (level as usize).saturating_sub(1);
                let unlocked = admin.all_levels || level == 1 || {
                    let prev_idx = idx.saturating_sub(1);
                    prev_idx < save_data.level_stars.len() && save_data.level_stars[prev_idx] > 0
                };
                if unlocked {
                    current_level.0 = level;
                    next_state.set(AppState::HeroSelect);
                }
            }
            MenuAction::BackToMenu => next_state.set(AppState::MainMenu),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Hero Select
// ---------------------------------------------------------------------------

fn setup_hero_select(
    mut commands: Commands,
    active_hero: Res<ActiveHeroType>,
    existing_cameras: Query<Entity, With<MenuCamera>>,
    asset_server: Res<AssetServer>,
    save: Option<Res<crate::save::SaveData>>,
    admin: Res<crate::resources::AdminUnlocks>,
) {
    // Use Camera3d for hero preview
    if existing_cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Msaa::Off,
            Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            MenuCamera,
            HeroPreviewRoot,
        ));
    }

    // Lighting for preview
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            shadows_enabled: !cfg!(target_os = "android"),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
        HeroPreviewRoot,
    ));
    commands.insert_resource(ClearColor(Color::srgb(0.05, 0.03, 0.1)));

    let save_data = save.map(|s| s.clone()).unwrap_or_default();

    // Spawn hero model preview (only if hero is unlocked)
    if is_hero_unlocked(active_hero.0, &save_data, &admin) {
        spawn_hero_preview(&mut commands, &asset_server, active_hero.0);
    }

    build_hero_select_screen(&mut commands, &active_hero, &save_data, &admin);

    // Admin panel
    spawn_admin_panel(&mut commands, &admin);
}

fn spawn_hero_preview(commands: &mut Commands, asset_server: &AssetServer, hero_type: crate::data::HeroType) {
    use crate::data::HeroType;
    let stats = crate::data::hero_stats(hero_type);
    let scene = asset_server.load(format!("{}#Scene0", stats.model_path));

    // Preview-specific overrides — game stats are tuned for the gameplay camera,
    // not the close-up hero select camera at (0, 2, 5).
    let (preview_scale, preview_y, preview_rot_y) = match hero_type {
        HeroType::IceHulk => (0.9, 0.3, 0.0),
        HeroType::NorthernOutsider => (0.012, 0.0, 0.0),
        HeroType::Pharaoh => (0.01, 0.2, 0.0),
        HeroType::ScarletMagus => (1.0, 0.0, 0.0),
        _ => (stats.model_scale, stats.model_y_offset, 0.0),
    };

    let mut transform = Transform::from_translation(Vec3::new(0.0, preview_y, 0.0))
        .with_scale(Vec3::splat(preview_scale));
    if stats.model_rotation_x != 0.0 {
        transform.rotate_x(stats.model_rotation_x);
    }
    if preview_rot_y != 0.0 {
        transform.rotate_y(preview_rot_y);
    }

    let mut entity_cmds = commands.spawn((
        SceneRoot(scene),
        transform,
        HeroPreviewModel,
        HeroPreviewRoot,
        crate::systems::showcase::ShowcaseNeedsAnim(
            format!("{}#Animation0", stats.idle_anim)
        ),
    ));

    // Northern Outsider needs curve stripping for rotation-only anims
    if stats.rotation_only_anims {
        let clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", stats.idle_anim));
        entity_cmds.insert(crate::systems::showcase::NeedsCurveStrip(clip));
    }
}

fn handle_hero_select(
    mut commands: Commands,
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut active_hero: ResMut<ActiveHeroType>,
    mut needs_setup: ResMut<crate::resources::NeedsFreshSetup>,
    roots: Query<Entity, With<MenuScreenRoot>>,
    preview_models: Query<Entity, With<HeroPreviewModel>>,
    asset_server: Res<AssetServer>,
    save: Option<Res<crate::save::SaveData>>,
    admin: Res<crate::resources::AdminUnlocks>,
) {
    let save_data = save.map(|s| s.clone()).unwrap_or_default();
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            MenuAction::SelectHero(hero_type) => {
                // Only allow selecting unlocked heroes
                if !is_hero_unlocked(hero_type, &save_data, &admin) {
                    continue;
                }
                active_hero.0 = hero_type;
                // Rebuild UI
                for entity in &roots {
                    commands.entity(entity).despawn_recursive();
                }
                // Despawn old preview model and spawn new one
                for entity in &preview_models {
                    commands.entity(entity).despawn_recursive();
                }
                spawn_hero_preview(&mut commands, &asset_server, hero_type);
                build_hero_select_screen(&mut commands, &active_hero, &save_data, &admin);
            }
            MenuAction::StartGame => {
                // Ensure selected hero is unlocked
                if !is_hero_unlocked(active_hero.0, &save_data, &admin) {
                    continue;
                }
                needs_setup.0 = true;
                next_state.set(AppState::Playing);
            }
            MenuAction::BackToLevelSelect => next_state.set(AppState::LevelSelect),
            _ => {}
        }
    }
}

fn build_hero_select_screen(
    commands: &mut Commands,
    active_hero: &ActiveHeroType,
    save_data: &crate::save::SaveData,
    admin: &crate::resources::AdminUnlocks,
) {
    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("Select Hero"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));

            // Hero cards — scrollable row pinned to bottom so 3D preview stays visible
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(4.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    max_height: Val::Percent(45.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(|row| {
                for hero_type in &crate::data::ALL_HERO_TYPES {
                    let info = crate::data::hero_info(*hero_type);
                    let stats = crate::data::hero_stats(*hero_type);
                    let selected = active_hero.0 == *hero_type;
                    let unlocked = is_hero_unlocked(*hero_type, save_data, admin);
                    let [r, g, b] = info.color;

                    let bg = if !unlocked {
                        Color::srgba(0.08, 0.08, 0.08, 0.7)
                    } else if selected {
                        Color::srgba(r * 0.3, g * 0.3, b * 0.3, 0.95)
                    } else {
                        Color::srgba(0.12, 0.08, 0.2, 0.9)
                    };

                    row.spawn((
                        Button,
                        MenuButton(MenuAction::SelectHero(*hero_type)),
                        Node {
                            width: Val::Px(120.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                            row_gap: Val::Px(1.0),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|card| {
                        if unlocked {
                            let hero_color = Color::srgb(r, g, b);
                            card.spawn((
                                Text::new(info.name),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(hero_color),
                            ));
                            card.spawn((
                                Text::new(info.role),
                                TextFont { font_size: 10.0, ..default() },
                                TextColor(Color::srgb(0.8, 0.8, 0.5)),
                            ));
                            card.spawn((
                                Text::new(format!("HP:{:.0} DMG:{:.0} SPD:{:.1}", stats.hp, stats.damage, stats.move_speed)),
                                TextFont { font_size: 9.0, ..default() },
                                TextColor(Color::srgb(0.5, 0.7, 0.5)),
                            ));
                            if selected {
                                card.spawn((
                                    Text::new("SELECTED"),
                                    TextFont { font_size: 10.0, ..default() },
                                    TextColor(Color::srgb(1.0, 1.0, 0.3)),
                                ));
                            }
                        } else {
                            // Locked hero
                            card.spawn((
                                Text::new(info.name),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(Color::srgb(0.4, 0.4, 0.4)),
                            ));
                            card.spawn((
                                Text::new("LOCKED"),
                                TextFont { font_size: 11.0, ..default() },
                                TextColor(Color::srgb(0.6, 0.3, 0.3)),
                            ));
                            let req = crate::data::hero_unlock_level(*hero_type);
                            card.spawn((
                                Text::new(format!("Beat Level {}", req)),
                                TextFont { font_size: 9.0, ..default() },
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            ));
                        }
                    });
                }
            });

            // Bottom buttons — always visible
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    padding: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
            ))
            .with_children(|row| {
                spawn_menu_button(
                    row, "Back",
                    MenuAction::BackToLevelSelect,
                    140.0,
                    Color::srgba(0.3, 0.15, 0.15, 0.9),
                    Color::srgb(1.0, 0.7, 0.7),
                );
                spawn_menu_button(
                    row, "Start!",
                    MenuAction::StartGame,
                    180.0,
                    Color::srgba(0.15, 0.35, 0.15, 0.95),
                    Color::WHITE,
                );
            });
        });
}

// ---------------------------------------------------------------------------
// Upgrade Shop
// ---------------------------------------------------------------------------

fn setup_upgrade_shop(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<MenuCamera>>,
    save: Option<Res<crate::save::SaveData>>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    let save_data = save.map(|s| s.clone()).unwrap_or_default();

    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Header with gem count
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            }).with_children(|header| {
                header.spawn((
                    Text::new("Upgrade Shop"),
                    TextFont { font_size: 36.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.8, 1.0)),
                ));
                header.spawn((
                    Text::new(format!("Gems: {}", save_data.aether_gems)),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));
            });

            // Upgrade cards
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(14.0),
                row_gap: Val::Px(14.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|row| {
                for &kind in &crate::data::ALL_UPGRADES {
                    let def = crate::data::upgrade_def(kind);
                    let idx = crate::data::upgrade_index(kind);
                    let current_level = if idx < save_data.upgrade_levels.len() {
                        save_data.upgrade_levels[idx]
                    } else { 0 };
                    let maxed = current_level >= crate::data::UPGRADE_MAX_LEVEL;
                    let cost = if maxed { 0 } else {
                        crate::data::UPGRADE_COSTS[current_level as usize]
                    };
                    let affordable = !maxed && save_data.aether_gems >= cost;

                    row.spawn((
                        Button,
                        MenuButton(MenuAction::BuyUpgrade(kind)),
                        Node {
                            width: Val::Px(200.0),
                            min_height: Val::Px(160.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(if affordable {
                            Color::srgba(0.1, 0.2, 0.35, 0.9)
                        } else if maxed {
                            Color::srgba(0.15, 0.25, 0.1, 0.8)
                        } else {
                            Color::srgba(0.1, 0.1, 0.15, 0.6)
                        }),
                        BorderRadius::all(Val::Px(10.0)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(def.name),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        card.spawn((
                            Text::new(def.description),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                        card.spawn((
                            Text::new(def.per_level),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::srgb(0.5, 0.8, 1.0)),
                        ));
                        // Level pips
                        let filled: String = (0..current_level).map(|_| '\u{25CF}').collect(); // filled circles
                        let empty: String = (0..(crate::data::UPGRADE_MAX_LEVEL - current_level)).map(|_| '\u{25CB}').collect();
                        card.spawn((
                            Text::new(format!("{}{}", filled, empty)),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::srgb(0.5, 0.8, 1.0)),
                        ));
                        // Cost or MAX
                        if maxed {
                            card.spawn((
                                Text::new("MAX"),
                                TextFont { font_size: 16.0, ..default() },
                                TextColor(Color::srgb(0.4, 0.8, 0.3)),
                            ));
                        } else {
                            card.spawn((
                                Text::new(format!("{} gems", cost)),
                                TextFont { font_size: 14.0, ..default() },
                                TextColor(if affordable {
                                    Color::srgb(1.0, 0.85, 0.3)
                                } else {
                                    Color::srgb(0.4, 0.4, 0.4)
                                }),
                            ));
                        }
                    });
                }
            });

            // Back button
            spawn_menu_button(
                root, "Back",
                MenuAction::BackToMenu,
                160.0,
                Color::srgba(0.3, 0.15, 0.15, 0.9),
                Color::srgb(1.0, 0.7, 0.7),
            );
        });
}

fn handle_upgrade_shop(
    mut commands: Commands,
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut save: Option<ResMut<crate::save::SaveData>>,
    roots: Query<Entity, With<MenuScreenRoot>>,
    cameras: Query<Entity, With<MenuCamera>>,
) {
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            MenuAction::BuyUpgrade(kind) => {
                let Some(save) = save.as_mut() else { continue };
                let idx = crate::data::upgrade_index(kind);
                let current_level = if idx < save.upgrade_levels.len() {
                    save.upgrade_levels[idx]
                } else { continue };

                if current_level >= crate::data::UPGRADE_MAX_LEVEL { continue; }
                let cost = crate::data::UPGRADE_COSTS[current_level as usize];
                if save.aether_gems < cost { continue; }

                // Purchase
                save.aether_gems -= cost;
                save.upgrade_levels[idx] = current_level + 1;

                // Persist
                let path = if cfg!(target_os = "android") {
                    "/data/data/com.agesofaether/files/save.json".to_string()
                } else {
                    "save.json".to_string()
                };
                if let Ok(json) = serde_json::to_string_pretty(&**save) {
                    let _ = std::fs::write(&path, json);
                }

                // Rebuild shop screen
                for entity in &roots {
                    commands.entity(entity).despawn_recursive();
                }
                let cam_exists = !cameras.is_empty();
                if !cam_exists {
                    commands.spawn((Camera2d, MenuCamera));
                }
                build_upgrade_shop_screen(&mut commands, save);
            }
            MenuAction::BackToMenu => next_state.set(AppState::MainMenu),
            _ => {}
        }
    }
}

fn build_upgrade_shop_screen(commands: &mut Commands, save: &crate::save::SaveData) {
    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            }).with_children(|header| {
                header.spawn((
                    Text::new("Upgrade Shop"),
                    TextFont { font_size: 36.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.8, 1.0)),
                ));
                header.spawn((
                    Text::new(format!("Gems: {}", save.aether_gems)),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(14.0),
                row_gap: Val::Px(14.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|row| {
                for &kind in &crate::data::ALL_UPGRADES {
                    let def = crate::data::upgrade_def(kind);
                    let idx = crate::data::upgrade_index(kind);
                    let current_level = if idx < save.upgrade_levels.len() {
                        save.upgrade_levels[idx]
                    } else { 0 };
                    let maxed = current_level >= crate::data::UPGRADE_MAX_LEVEL;
                    let cost = if maxed { 0 } else {
                        crate::data::UPGRADE_COSTS[current_level as usize]
                    };
                    let affordable = !maxed && save.aether_gems >= cost;

                    row.spawn((
                        Button,
                        MenuButton(MenuAction::BuyUpgrade(kind)),
                        Node {
                            width: Val::Px(200.0),
                            min_height: Val::Px(160.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(if affordable {
                            Color::srgba(0.1, 0.2, 0.35, 0.9)
                        } else if maxed {
                            Color::srgba(0.15, 0.25, 0.1, 0.8)
                        } else {
                            Color::srgba(0.1, 0.1, 0.15, 0.6)
                        }),
                        BorderRadius::all(Val::Px(10.0)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(def.name),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        card.spawn((
                            Text::new(def.description),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                        card.spawn((
                            Text::new(def.per_level),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::srgb(0.5, 0.8, 1.0)),
                        ));
                        let filled: String = (0..current_level).map(|_| '\u{25CF}').collect();
                        let empty: String = (0..(crate::data::UPGRADE_MAX_LEVEL - current_level)).map(|_| '\u{25CB}').collect();
                        card.spawn((
                            Text::new(format!("{}{}", filled, empty)),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::srgb(0.5, 0.8, 1.0)),
                        ));
                        if maxed {
                            card.spawn((
                                Text::new("MAX"),
                                TextFont { font_size: 16.0, ..default() },
                                TextColor(Color::srgb(0.4, 0.8, 0.3)),
                            ));
                        } else {
                            card.spawn((
                                Text::new(format!("{} gems", cost)),
                                TextFont { font_size: 14.0, ..default() },
                                TextColor(if affordable {
                                    Color::srgb(1.0, 0.85, 0.3)
                                } else {
                                    Color::srgb(0.4, 0.4, 0.4)
                                }),
                            ));
                        }
                    });
                }
            });

            spawn_menu_button(
                root, "Back",
                MenuAction::BackToMenu,
                160.0,
                Color::srgba(0.3, 0.15, 0.15, 0.9),
                Color::srgb(1.0, 0.7, 0.7),
            );
        });
}

// ---------------------------------------------------------------------------
// Logbook
// ---------------------------------------------------------------------------

fn setup_logbook(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<MenuCamera>>,
) {
    if existing_cameras.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Logbook"),
                TextFont { font_size: 40.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            // Tab buttons
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ))
            .with_children(|row| {
                spawn_menu_button(row, "Enemies", MenuAction::LogbookEnemies, 140.0, Color::srgba(0.25, 0.1, 0.1, 0.9), Color::srgb(1.0, 0.7, 0.7));
                spawn_menu_button(row, "Towers", MenuAction::LogbookTowers, 140.0, Color::srgba(0.1, 0.1, 0.25, 0.9), Color::srgb(0.7, 0.7, 1.0));
            });

            // Default: show enemies page
            spawn_logbook_enemies(root);

            // Back button
            spawn_menu_button(
                root, "Back",
                MenuAction::BackToMenu,
                160.0,
                Color::srgba(0.3, 0.15, 0.15, 0.9),
                Color::srgb(1.0, 0.7, 0.7),
            );
        });
}

fn spawn_logbook_enemies(parent: &mut ChildBuilder) {
    parent.spawn((
        LogbookPageRoot,
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(10.0),
            row_gap: Val::Px(10.0),
            justify_content: JustifyContent::Center,
            max_width: Val::Px(900.0),
            ..default()
        },
    ))
    .with_children(|grid| {
        for enemy_type in &crate::data::ALL_ENEMY_TYPES {
            let info = crate::data::enemy_info(*enemy_type);
            let stats = crate::data::enemy_stats(*enemy_type);

            grid.spawn((
                Node {
                    width: Val::Px(200.0),
                    min_height: Val::Px(90.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(3.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.08, 0.18, 0.9)),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(info.name),
                    TextFont { font_size: 17.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                card.spawn((
                    Text::new(info.traits),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.7, 0.3)),
                ));
                card.spawn((
                    Text::new(info.description),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
                card.spawn((
                    Text::new(format!("HP: {:.0}  SPD: {:.1}  ARM: {:.0}", stats.hp, stats.speed, stats.armor)),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.7, 0.5)),
                ));
            });
        }
    });
}

fn spawn_logbook_towers(parent: &mut ChildBuilder) {
    parent.spawn((
        LogbookPageRoot,
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(10.0),
            row_gap: Val::Px(10.0),
            justify_content: JustifyContent::Center,
            max_width: Val::Px(900.0),
            ..default()
        },
    ))
    .with_children(|grid| {
        let elements = [Element::Lightning, Element::Earth, Element::Ice, Element::Fire];
        for element in &elements {
            let base = tower_stats(*element, 0);
            let color = element_color(*element);

            grid.spawn((
                Node {
                    width: Val::Px(200.0),
                    min_height: Val::Px(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(3.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.08, 0.18, 0.9)),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(base.name),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(color),
                ));
                card.spawn((
                    Text::new(format!("Cost: {}g  DMG: {:.0}  RNG: {:.1}", base.cost, base.damage, base.range)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
                // Specializations
                let specs = element_specializations(*element);
                for (_, spec_def) in &specs {
                    card.spawn((
                        Text::new(format!("{} — {}", spec_def.name, spec_def.description)),
                        TextFont { font_size: 10.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.7)),
                    ));
                }
            });
        }
    });
}

fn handle_logbook(
    mut commands: Commands,
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
    pages: Query<Entity, With<LogbookPageRoot>>,
    roots: Query<Entity, With<MenuScreenRoot>>,
) {
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            MenuAction::BackToMenu | MenuAction::LogbookBack => {
                next_state.set(AppState::MainMenu);
            }
            MenuAction::LogbookEnemies => {
                // Remove current page and spawn enemies
                for entity in &pages {
                    commands.entity(entity).despawn_recursive();
                }
                if let Ok(root_entity) = roots.get_single() {
                    commands.entity(root_entity).with_children(|root| {
                        spawn_logbook_enemies(root);
                    });
                }
            }
            MenuAction::LogbookTowers => {
                for entity in &pages {
                    commands.entity(entity).despawn_recursive();
                }
                if let Ok(root_entity) = roots.get_single() {
                    commands.entity(root_entity).with_children(|root| {
                        spawn_logbook_towers(root);
                    });
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Credits
// ---------------------------------------------------------------------------

fn setup_credits(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<MenuCamera>>,
) {
    commands.insert_resource(ClearColor(Color::srgb(0.05, 0.02, 0.1)));

    if existing_cameras.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    commands
        .spawn((
            MenuScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("Credits"),
                TextFont { font_size: 42.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
                Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
            ));

            // Scrollable content area
            root.spawn(Node {
                width: Val::Percent(90.0),
                max_width: Val::Px(600.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            }).with_children(|scroll| {
                // Game
                credits_section(scroll, "Game Design & Development", &[
                    "Ages of Aether",
                ]);

                // 3D Models
                credits_section(scroll, "3D Models", &[
                    "Colossus pre Tilt Brush — Darwin Yamamoto [CC-BY] via Poly Pizza",
                    "Hive Turret — Zsky [CC-BY] via Poly Pizza",
                    "Low Poly Giant Sloth — rkuhlf [CC-BY] via Sketchfab",
                    "Low Poly Mammoth — rkuhlf [CC-BY] via Sketchfab",
                    "Saber-toothed Cat [ai] — dinoguy263allo [CC-BY] via Sketchfab",
                    "Volcano — Poly by Google [CC-BY] via Poly Pizza",
                    "White Eagle Animation — GremorySaiyan [CC-BY] via Sketchfab",
                    "Roman Legionary — 3dUVpro [CC-BY] via Sketchfab",
                    "Woolly Rhino — Raven-Woods [CC-BY] via Sketchfab",
                    "Low Poly Knight — Pascal T. Monette [CC-BY] via Sketchfab",
                    "Toon Horse with Saddle — flairetic [CC-BY] via Sketchfab",
                    "Colossus — MASTER MODS [CC-BY] via Sketchfab",
                    "Full Rig Lion 2 — TC5051 [CC-BY] via Sketchfab",
                    "Oliphaunt — Josiah Miller [CC-BY] via Sketchfab",
                    "Dodo — BlueMesh [CC-BY] via Sketchfab",
                    "Giant Blue Hulk Mutant Beast — Ethan C [CC-BY] via Sketchfab",
                    "Northern Outsider — Splodeman [CC-BY] via Sketchfab",
                    "Storm — MIKESTEEZ [CC-BY] via Sketchfab",
                    "Pharaoh X-suit — YT-XTREMENINJA [CC-BY] via Sketchfab",
                    "Castle — CreativeTrio6 [CC-BY] via Poly Pizza",
                    "Coliseum — Poly by Google [CC-BY] via Poly Pizza",
                ]);

                // Animation
                credits_section(scroll, "Animation", &[
                    "Character animations via Mixamo by Adobe",
                ]);

                // Built with
                credits_section(scroll, "Built With", &[
                    "Bevy Engine \u{2022} Rust",
                ]);

                // Thanks
                scroll.spawn((
                    Text::new("Thank you for playing!"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.8, 0.5)),
                    Node {
                        margin: UiRect::vertical(Val::Px(12.0)),
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                ));
            });

            // Back button
            spawn_menu_button(
                root, "Back",
                MenuAction::BackToMenu,
                200.0,
                Color::srgba(0.2, 0.15, 0.4, 0.9),
                Color::WHITE,
            );
        });
}

fn credits_section(parent: &mut ChildBuilder, title: &str, entries: &[&str]) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        ..default()
    }).with_children(|section| {
        // Section title
        section.spawn((
            Text::new(title),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.7, 0.6, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
        ));
        // Entries
        for entry in entries {
            section.spawn((
                Text::new(*entry),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));
        }
    });
}

fn handle_credits(
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }
        if matches!(btn.0, MenuAction::BackToMenu) {
            next_state.set(AppState::MainMenu);
        }
    }
}

// ---------------------------------------------------------------------------
// Admin Panel (level select & hero select)
// ---------------------------------------------------------------------------

fn spawn_admin_panel(commands: &mut Commands, admin: &crate::resources::AdminUnlocks) {
    commands.spawn((
        AdminPanelRoot,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            bottom: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        BorderRadius::all(Val::Px(6.0)),
        GlobalZIndex(20),
    )).with_children(|panel| {
        panel.spawn((
            Text::new("Admin"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(1.0, 0.4, 0.4)),
        ));

        let levels_label = if admin.all_levels { "Levels: ON" } else { "Unlock Levels" };
        let levels_color = if admin.all_levels { Color::srgb(0.4, 1.0, 0.4) } else { Color::srgb(0.8, 0.6, 1.0) };
        spawn_admin_button(panel, levels_label, levels_color, AdminUnlockLevelsButton);

        let heroes_label = if admin.all_heroes { "Heroes: ON" } else { "Unlock Heroes" };
        let heroes_color = if admin.all_heroes { Color::srgb(0.4, 1.0, 0.4) } else { Color::srgb(1.0, 0.85, 0.4) };
        spawn_admin_button(panel, heroes_label, heroes_color, AdminUnlockHeroesButton);
    });
}

fn spawn_admin_button<M: Component>(parent: &mut ChildBuilder, label: &str, text_color: Color, marker: M) {
    parent.spawn((
        Button,
        marker,
        Node {
            width: Val::Px(130.0),
            height: Val::Px(28.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
        BorderRadius::all(Val::Px(4.0)),
    )).with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont { font_size: 12.0, ..default() },
            TextColor(text_color),
        ));
    });
}

fn handle_admin_panel(
    mut commands: Commands,
    levels_q: Query<&Interaction, (Changed<Interaction>, With<AdminUnlockLevelsButton>)>,
    heroes_q: Query<&Interaction, (Changed<Interaction>, With<AdminUnlockHeroesButton>)>,
    mut admin: ResMut<crate::resources::AdminUnlocks>,
    admin_panels: Query<Entity, With<AdminPanelRoot>>,
    roots: Query<Entity, With<MenuScreenRoot>>,
    state: Res<State<AppState>>,
    save: Option<Res<crate::save::SaveData>>,
    active_hero: Res<ActiveHeroType>,
    preview_models: Query<Entity, With<HeroPreviewModel>>,
    asset_server: Res<AssetServer>,
) {
    let mut changed = false;
    for interaction in &levels_q {
        if *interaction == Interaction::Pressed {
            admin.all_levels = !admin.all_levels;
            info!("ADMIN: all levels unlocked = {}", admin.all_levels);
            changed = true;
        }
    }
    for interaction in &heroes_q {
        if *interaction == Interaction::Pressed {
            admin.all_heroes = !admin.all_heroes;
            info!("ADMIN: all heroes unlocked = {}", admin.all_heroes);
            changed = true;
        }
    }
    if changed {
        // Rebuild admin panel
        for entity in &admin_panels {
            commands.entity(entity).despawn_recursive();
        }
        spawn_admin_panel(&mut commands, &admin);
        // Rebuild the menu screen
        for entity in &roots {
            commands.entity(entity).despawn_recursive();
        }
        let save_data = save.map(|s| s.clone()).unwrap_or_default();
        match state.get() {
            AppState::LevelSelect => {
                build_level_select_screen(&mut commands, &save_data, &admin);
            }
            AppState::HeroSelect => {
                // Rebuild preview if heroes toggled
                for entity in &preview_models {
                    commands.entity(entity).despawn_recursive();
                }
                if is_hero_unlocked(active_hero.0, &save_data, &admin) {
                    spawn_hero_preview(&mut commands, &asset_server, active_hero.0);
                }
                build_hero_select_screen(&mut commands, &active_hero, &save_data, &admin);
            }
            _ => {}
        }
    }
}

/// Returns true if a hero is unlocked based on save data or admin overrides.
fn is_hero_unlocked(
    hero: crate::data::HeroType,
    save: &crate::save::SaveData,
    admin: &crate::resources::AdminUnlocks,
) -> bool {
    if admin.all_heroes { return true; }
    let required = crate::data::hero_unlock_level(hero);
    if required == 0 { return true; }
    let idx = (required as usize).saturating_sub(1);
    idx < save.level_stars.len() && save.level_stars[idx] > 0
}
