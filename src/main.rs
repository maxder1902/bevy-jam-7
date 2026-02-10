// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod menus;
mod screens;
mod theme;

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::{
    asset::AssetMetaCheck,
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    light::{AtmosphereEnvironmentMapLight, GlobalAmbientLight, SunDisk, VolumetricFog},
    pbr::{Atmosphere, AtmosphereSettings, Falloff, PhaseFunction, ScatteringTerm},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevy_skein::SkeinPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Set black clear color so atmosphere is visible
        app.insert_resource(ClearColor(Color::BLACK));
        // Disable ambient light - atmosphere will provide lighting
        app.insert_resource(GlobalAmbientLight::NONE);

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy New 2D".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        );

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
            SkeinPlugin::default(),
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
        app.add_systems(
            Update,
            |mut physics_time: ResMut<Time<Physics>>, paused: Res<State<Pause>>| {
                if paused.0 {
                    physics_time.pause();
                } else {
                    physics_time.unpause();
                }
            },
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<bevy::pbr::ScatteringMedium>>,
) {
    // Custom alien green atmosphere with green-dominant Rayleigh scattering
    let green_medium = bevy::pbr::ScatteringMedium::new(
        256,
        256,
        [
            // Rayleigh scattering term - green-purple
            ScatteringTerm {
                absorption: Vec3::ZERO,
                scattering: Vec3::new(10.0e-6, 80.0e-6, 15.0e-6), // Much higher green, reduced red/blue
                falloff: Falloff::Exponential { scale: 8.0 / 60.0 },
                phase: PhaseFunction::Rayleigh,
            },
            // Mie scattering term - thick atmosphere (2.0e-6)
            ScatteringTerm {
                absorption: Vec3::splat(3.996e-6),
                scattering: Vec3::splat(2.0e-6), // Thick Mie scattering
                falloff: Falloff::Exponential { scale: 1.2 / 60.0 },
                phase: PhaseFunction::Mie { asymmetry: 0.8 },
            },
        ],
    );

    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Atmosphere::earthlike(scattering_mediums.add(green_medium)),
        AtmosphereSettings::default(),
        Exposure {
            ev100: Exposure::EV100_BLENDER,
        },
        Tonemapping::AcesFitted,
        // Without bloom sun is just white circle
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
        VolumetricFog::default(),
    ));
}
