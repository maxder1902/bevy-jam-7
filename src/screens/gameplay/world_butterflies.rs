use bevy::prelude::*;
use bevy_mesh::morph::MorphWeights;
use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::screens::Screen;

/* ------------------------------------------------ */
/* ---------------- COMPONENTES ------------------- */
/* ------------------------------------------------ */

#[derive(Component)]
pub struct Butterfly {
    pub flap_speed: f32,
    pub phase_offset: f32,
}

#[derive(Component)]
pub struct ButterflyMovement {
    pub target: Vec3,
    pub speed: f32,
    pub change_target_timer: Timer,
    pub base_position: Vec3,
}

/* ------------------------------------------------ */
/* ---------------- RECURSOS ---------------------- */
/* ------------------------------------------------ */

#[derive(Resource, Deref, DerefMut)]
struct ButterflyAnimTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct ButterflyRng(SmallRng);

/* ------------------------------------------------ */
/* ---------------- PLUGIN ------------------------ */
/* ------------------------------------------------ */

pub struct WorldButterfliesPlugin;

impl Plugin for WorldButterfliesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ButterflyAnimTimer(Timer::from_seconds(
            0.05,
            TimerMode::Repeating,
        )))
        .insert_resource(ButterflyRng(SmallRng::from_rng(&mut rand::rng())))
        .add_systems(OnEnter(Screen::Gameplay), spawn_butterflies)
        .add_systems(
            Update,
            (
                detect_butterfly_meshes,
                animate_butterfly_wings,
                move_butterflies,
            )
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

fn spawn_butterflies(asset_server: Res<AssetServer>, mut commands: Commands) {
    let handle: Handle<Scene> = asset_server.load(
        GltfAssetLabel::Scene(1).from_asset("models/Demo_level_heaven_sword.glb"),
    );

    info!("🦋 Butterfly System: Large Map Mode (240x240m)");

    commands.spawn((
        SceneRoot(handle),
        Transform::default(),
        Visibility::default(),
    ));
}

/* ------------------------------------------------ */
/* --------- DETECCIÓN (LÍMITE 5 UNIDADES) -------- */
/* ------------------------------------------------ */

fn detect_butterfly_meshes(
    mut commands: Commands,
    morph_query: Query<(Entity, &Name), (Without<Butterfly>, Added<MorphWeights>)>,
    empty_query: Query<(Entity, &Name, &GlobalTransform), (Without<ButterflyMovement>, Added<Name>)>,
) {
    // 1. Setup de Meshes (Shape Keys) - Limitamos a 5
    let mut count = 0;
    for (entity, name) in &morph_query {
        if name.as_str().to_lowercase().contains("butterfly") && count < 5 {
            commands.entity(entity).insert((
                Butterfly {
                    flap_speed: rand::random_range(10.0..18.0),
                    phase_offset: rand::random_range(0.0..std::f32::consts::TAU),
                },
                Transform::from_scale(Vec3::splat(rand::random_range(1.0..1.5))),
            ));
            count += 1;
        }
    }

    // 2. Setup de Movimiento - Limitamos a 5
    let mut move_count = 0;
    for (entity, name, global_transform) in &empty_query {
        if name.as_str().to_lowercase().contains("butterfly") && move_count < 5 {
            let pos = global_transform.translation();

            commands.entity(entity).insert(ButterflyMovement {
                base_position: pos,
                // Dispersión inicial masiva para mapa de 240m
                target: pos + Vec3::new(
                    rand::random_range(-80.0..80.0),
                    rand::random_range(5.0..25.0),
                    rand::random_range(-80.0..80.0),
                ),
                speed: rand::random_range(4.0..8.0), // Velocidad aumentada para distancias largas
                change_target_timer: Timer::from_seconds(
                    rand::random_range(5.0..12.0), // Tiempos de vuelo más largos
                    TimerMode::Repeating,
                ),
            });
            move_count += 1;
        }
    }
}

/* ------------------------------------------------ */
/* ----------- ANIMACIÓN ALAS (Shape Keys) -------- */
/* ------------------------------------------------ */

fn animate_butterfly_wings(
    time: Res<Time>,
    mut timer: ResMut<ButterflyAnimTimer>,
    mut query: Query<(&Butterfly, &mut MorphWeights)>,
) {
    timer.tick(time.delta());
    if !timer.just_finished() { return; }

    let t = time.elapsed_secs();
    for (butterfly, mut morph) in &mut query {
        let v = ((t * butterfly.flap_speed + butterfly.phase_offset).sin() + 1.0) * 0.5;
        for w in morph.weights_mut().iter_mut() {
            *w = v;
        }
    }
}

/* ------------------------------------------------ */
/* --------------- MOVIMIENTO MASIVO -------------- */
/* ------------------------------------------------ */

fn move_butterflies(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ButterflyMovement)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut movement) in &mut query {
        movement.change_target_timer.tick(time.delta());

        // Cuando cambian de rumbo, eligen un punto muy lejano en el mapa
        if movement.change_target_timer.just_finished() {
            movement.target = movement.base_position + Vec3::new(
                rand::random_range(-110.0..110.0), // Casi la mitad del mapa (240m)
                rand::random_range(-10.0..30.0),   // Variación de altura notable
                rand::random_range(-110.0..110.0),
            );
        }

        let direction = (movement.target - transform.translation).normalize_or_zero();

        if direction.length_squared() > 0.001 {
            // Mover hacia el objetivo lejano
            transform.translation += direction * movement.speed * dt;

            // Rotación suave para grandes trayectorias
            let target_rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            transform.rotation = transform.rotation.slerp(target_rotation, dt * 1.2);
        }
    }
}
