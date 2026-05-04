use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

pub mod components;
pub mod data;
pub mod resources;
pub mod save;
pub mod states;
pub mod systems;
pub mod ui;

#[bevy_main]
pub fn main() {
    let mut app = App::new();

    #[cfg(target_os = "android")]
    {
        use bevy::winit::WinitSettings;
        use bevy::window::WindowMode;
        app.add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ages of Aether".into(),
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }));
        app.insert_resource(WinitSettings::mobile());
    }

    #[cfg(not(target_os = "android"))]
    app.add_plugins(DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ages of Aether".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        })
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }));

    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_state::<states::AppState>()
        .init_resource::<resources::GameData>()
        .init_resource::<resources::WaveState>()
        .init_resource::<resources::Selection>()
        .init_resource::<resources::WaveButtonPressed>()
        .init_resource::<resources::AutoWave>()
        .init_resource::<resources::GameSpeed>()
        .init_resource::<resources::NeedsFreshSetup>()
        .init_resource::<resources::HeroMoveCommand>()
        .init_resource::<resources::ActiveHeroType>()
        .init_resource::<resources::NoHeroSelected>()
        .init_resource::<resources::NewlyUnlockedHero>()
        .init_resource::<resources::AdminUnlocks>()
        .init_resource::<resources::CurrentLevel>()
        .init_resource::<resources::Difficulty>()
        .init_resource::<resources::LevelPath>()
        .init_resource::<resources::VolumeSettings>()
        .init_resource::<states::GameOutcome>()
        .add_systems(Startup, save::load_save_on_startup)
        .add_systems(OnEnter(states::AppState::GameOver), save::save_on_level_complete)
        .init_resource::<systems::CameraFocus>()
        .init_resource::<systems::camera::CameraShake>()
        .init_resource::<systems::camera::CameraIntro>()
        .init_resource::<systems::debug::DebugState>()
        .init_resource::<systems::debug::AdminMode>()
        .init_resource::<systems::hero_ability::AbilityActivated>()
        .init_resource::<systems::tower_spec::SpecializationChosen>()
        .init_resource::<systems::tower_spec::SpecUpgradeRequested>()
        .init_resource::<resources::PlayerAbilities>()
        .init_resource::<resources::PlayerAbilityTargeting>()
        .init_resource::<systems::tutorial::TutorialState>()
        .add_plugins(systems::GamePlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}
