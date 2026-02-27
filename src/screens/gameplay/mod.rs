//! The screen state for the main gameplay.

use avian3d::prelude::*;
use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Exposure,
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    input::common_conditions::input_just_pressed,
    post_process::bloom::Bloom,
    prelude::*,
    window::CursorOptions,
};
use bevy_landmass::prelude::*;
use bevy_rerecast::prelude::*;
use bevy_seedling::sample::AudioSample;
use landmass_rerecast::{Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d};
use std::f32::consts::PI;

use crate::{
    PausableSystems, Pause,
    asset_tracking::LoadResource,
    menus::Menu,
    screens::{
        Screen,
        gameplay::{
            character_controller::CameraRotation,
            hammerhead::HammerheadAssets,
            katana::{katana_animation, katana_setup, poor_setup_for_katana_animations},
            player::Player,
            events::SpawnAlarmClockEvent,
        },
        set_cursor_grab,
    },
};

mod character_controller;
mod checkpoints;
mod enemy;
mod hammerhead;
mod katana;
mod player;
mod enemy_spawn;
mod world_butterflies;
mod spawn_enemy_waves;
mod hide_colliders;
mod hud;
mod flower_capsule;
mod fall_death;
mod cloud_goop;
mod alarm_clock;
mod events;
mod particle_system;

#[derive(Component)]
struct Level;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        PhysicsPlugins::default(),
        bevy_landmass::Landmass3dPlugin::default(),
        bevy_landmass::debug::Landmass3dDebugPlugin::default(),
        bevy_rerecast::NavmeshPlugins::default(),
        avian_rerecast::AvianBackendPlugin::default(),
        LandmassRerecastPlugin::default(),
        character_controller::CharacterControllerPlugin,
        enemy::EnemyPlugin,
        enemy_spawn::EnemySpawnPlugin,
        world_butterflies::WorldButterfliesPlugin,
        checkpoints::CheckpointPlugin,
        spawn_enemy_waves::WaveSpawnPlugin,
        hide_colliders::HideCollidersPlugin,
        hammerhead::hammerhead,
        hud::HudPlugin,
    ));

    app.add_plugins((
        flower_capsule::FlowerCapsulePlugin,
        fall_death::FallDeathPlugin,
        cloud_goop::CloudGoopPlugin,
        alarm_clock::AlarmClockPlugin,
        particle_system::ParticleSystemPlugin,
    ));

    app.load_resource::<LevelAssets>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (setup_clock_spawn_resources, spawn_level, katana_setup).chain(),
    );
    app.add_systems(
        OnExit(Screen::Gameplay),
        (
            |mut commands: Commands, camera: Single<Entity, With<Camera3d>>| {
                commands
                    .entity(*camera)
                    .remove::<Skybox>()
                    .despawn_children()
                    .remove_parent_in_place();
            },
            cleanup_clock_spawn_resources,
        ),
    );

    app.add_systems(
        Update,
        (
            (pause, spawn_background_overlay, open_pause_menu).run_if(
                in_state(Screen::Gameplay)
                    .and(in_state(Menu::None))
                    .and(input_just_pressed(KeyCode::Escape)),
            ),
            go_to_death_menu.run_if(in_state(Screen::Gameplay).and(in_state(Menu::None))),
        ),
    );
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(
        Update,
        generate_navmesh.run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(handle_navmesh_ready);

    app.add_systems(
        Update,
        (
            poor_setup_for_katana_animations,
            katana_animation
                .in_set(PausableSystems)
                .run_if(in_state(Screen::Gameplay)),
            debug_spawn_clock
                .run_if(in_state(Screen::Gameplay)),
            cache_clock_spawn_points.run_if(in_state(Screen::Gameplay)),
            spawn_clocks_from_empties.run_if(in_state(Screen::Gameplay)),
        ),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSample>,
    // --- pasos ---
    #[dependency]
    step1: Handle<AudioSample>,
    #[dependency]
    pub step_stone: Handle<AudioSample>,
    #[dependency]
    pub stepping_crystal: Handle<AudioSample>,
    // --- combate enemigos ---
    #[dependency]
    pub hit_enemy_first: Handle<AudioSample>,
    #[dependency]
    pub hit_enemy_final: Handle<AudioSample>,
    // --- cápsula ---
    #[dependency]
    pub capsule_damage: Handle<AudioSample>,
    // --- armas / misc ---
    #[dependency]
    whoosh1: Handle<AudioSample>,
    // --- escenas / imágenes ---
    #[dependency]
    demo_level: Handle<Scene>,
    #[dependency]
    skybox: Handle<Image>,
    #[dependency]
    katana_idle: Handle<AnimationClip>,
    #[dependency]
    katana_swing: Handle<AnimationClip>,
    #[dependency]
    katana_scene: Handle<Scene>,
    #[dependency]
    hammerhead: HammerheadAssets,
    #[dependency]
    alarm_clock_scene: Handle<Scene>,
    #[dependency]
    time_field_scene: Handle<Scene>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            // pasos
            step1: assets.load("audio/sound_effects/step1.wav"),
            step_stone: assets.load("audio/sound_effects/step_stone_4.wav"),
            stepping_crystal: assets.load("audio/sound_effects/stepping-crystal.wav"),
            // combate
            hit_enemy_first: assets.load("audio/sound_effects/first-hit-2-enemy.wav"),
            hit_enemy_final: assets.load("audio/sound_effects/final-hit-2-enemy.wav"),
            // cápsula
            capsule_damage: assets.load("audio/sound_effects/capsule_damage.wav"),
            // misc
            whoosh1: assets.load("audio/sound_effects/whoosh1.wav"),
            // escenas
            demo_level: assets
                .load(GltfAssetLabel::Scene(1).from_asset("models/Demo_level_heaven_sword.glb")),
            skybox: assets.load("images/skybox.ktx2"),
            katana_idle: assets.load(GltfAssetLabel::Animation(0).from_asset("models/katana.glb")),
            katana_swing: assets.load(GltfAssetLabel::Animation(1).from_asset("models/katana.glb")),
            katana_scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/katana.glb")),
            alarm_clock_scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/alarm_clock.glb")),
            hammerhead: HammerheadAssets::load(assets),
            time_field_scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/time_field.glb")),
        }
    }
}

fn go_to_death_menu(
    mut commands: Commands,
    mut next_menu: ResMut<NextState<Menu>>,
    mut paused: ResMut<NextState<Pause>>,
    player: Single<&Player>,
) {
    if !player.is_alive() {
        commands.run_system_cached(spawn_background_overlay);
        next_menu.set(Menu::Death);
        paused.set(Pause(true));
    }
}

fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    camera: Single<Entity, With<Camera3d>>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut generator: NavmeshGenerator,
) {
    commands.insert_resource(NavmeshDone(false));
    let camera = *camera;

    let archipelago_options: ArchipelagoOptions<ThreeD> =
        ArchipelagoOptions::from_agent_radius(0.5);

    let archipelago_id = commands.spawn(Archipelago3d::new(archipelago_options)).id();

    commands.spawn(Island3dBundle {
        island: Island,
        archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
        nav_mesh: NavMeshHandle3d(generator.generate(NavmeshSettings {
            agent_radius: 0.5,
            ..default()
        })),
    });

    commands.insert_resource(NavmeshArchipelagoHolder(archipelago_id));

    set_cursor_grab(&mut cursor_options, true);
    let player = player::spawn_player(&mut commands, camera);

    let music = commands
        .spawn((
            Name::new("Gameplay Music"),
            // music(level_assets.music.clone()),
        ))
        .id();

    let transform = Transform::from_xyz(0.0, 0.8 + 0.9, 0.0);
    commands.entity(camera).insert((
        transform,
        CameraRotation(transform.rotation.x),
        Skybox {
            image: level_assets.skybox.clone(),
            brightness: 1000.0,
            ..Default::default()
        },
    ));

    let light = commands
        .spawn((
            Name::new("Light"),
            DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(
                EulerRot::YXZ,
                -35f32.to_radians(),
                -25f32.to_radians(),
                0.0,
            )),
        ))
        .id();

    let _level = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            SceneRoot(level_assets.demo_level.clone()),
            Level,
        ))
        .add_children(&[player, light, music])
        .id();
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(false));
}

fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(true));
}

fn spawn_background_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Background Overlay"),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        DespawnOnExit(Pause(true)),
    ));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

#[derive(Resource)]
struct NavmeshDone(bool);

#[derive(Resource)]
pub struct NavmeshArchipelagoHolder(pub Entity);

#[derive(Resource, Default)]
struct ClockSpawnPoints(Vec<Vec3>);

#[derive(Resource)]
struct ClockSpawnTimer(Timer);

fn generate_navmesh(
    mut generator: NavmeshGenerator,
    island: Query<&NavMeshHandle3d, With<Island>>,
    navmesh_done: Res<NavmeshDone>,
) {
    if navmesh_done.0 { return; }
    info!("Generating navmesh...");
    let mut count = 0;
    for island in &island {
        count += 1;
        generator.regenerate(&island.0, NavmeshSettings { agent_radius: 0.5, ..default() });
    }
    info!("Started navmesh regen for {count} islands");
}

fn handle_navmesh_ready(_: On<NavmeshReady>, mut navmesh_done: ResMut<NavmeshDone>) {
    info!("Navmesh ready!");
    navmesh_done.0 = true;
}

fn debug_spawn_clock(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        alarm_clock::spawn_alarm_clock(&mut commands, &level_assets, Vec3::new(0.0, 2.0, -3.0));
        info!("DEBUG: Reloj spawneado!");
    }
}

fn setup_clock_spawn_resources(mut commands: Commands) {
    commands.insert_resource(ClockSpawnPoints::default());
    let mut timer = Timer::from_seconds(20.0, TimerMode::Repeating);
    timer.set_elapsed(timer.duration());
    commands.insert_resource(ClockSpawnTimer(timer));
}

fn cleanup_clock_spawn_resources(mut commands: Commands) {
    commands.remove_resource::<ClockSpawnPoints>();
    commands.remove_resource::<ClockSpawnTimer>();
}

fn cache_clock_spawn_points(
    mut points: ResMut<ClockSpawnPoints>,
    named: Query<(&Name, &GlobalTransform), Added<GlobalTransform>>,
) {
    for (name, transform) in named.iter() {
        if name.as_str() == "SpawnAlarmClock" {
            points.0.push(transform.translation());
            info!("Spawn point de reloj registrado en {:?}", transform.translation());
        }
    }
}

fn spawn_clocks_from_empties(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    time: Res<Time>,
    mut timer: ResMut<ClockSpawnTimer>,
    points: Res<ClockSpawnPoints>,
) {
    if points.0.is_empty() { return; }
    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }
    for pos in points.0.iter().copied() {
        alarm_clock::spawn_alarm_clock(&mut commands, &level_assets, pos);
    }
    info!("Relojes spawneados desde {} empties", points.0.len());
}
