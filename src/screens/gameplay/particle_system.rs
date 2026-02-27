//! Sistema de partículas — nubes Big_Bad_Cloud y enemigos Hammerhead
//!
//! Reutiliza el sistema de estrellas de alarm_clock adaptado con
//! colores oscuros (negro, vino, rojo/morado muy oscuro).
//!
//! - Nubes: chispas oscuras que emanan constantemente (detectadas por CloudGoopAnimated)
//! - Enemigos: partículas idle + burst al recibir Knockback (detectados por Enemy component)

use bevy::prelude::*;
use rand::RngExt;
use crate::screens::Screen;
use crate::screens::gameplay::enemy::{Enemy, Knockback};
use crate::screens::gameplay::cloud_goop::CloudGoopAnimated;

pub struct ParticleSystemPlugin;

const COLOR_BLACK:       (f32, f32, f32) = (0.02, 0.0,  0.02);
const COLOR_WINE:        (f32, f32, f32) = (0.25, 0.0,  0.05);
const COLOR_DARK_PURPLE: (f32, f32, f32) = (0.15, 0.0,  0.2);

const EMISSIVE_CLOUD:  (f32, f32, f32) = (0.4, 0.0, 0.3);
const EMISSIVE_ENEMY:  (f32, f32, f32) = (0.6, 0.0, 0.1);

const CLOUD_SPAWN_INTERVAL: f32  = 0.15;
const CLOUD_PARTICLE_COUNT: usize = 3;
const ENEMY_IDLE_INTERVAL: f32   = 0.2;
const ENEMY_IDLE_COUNT: usize    = 2;
const ENEMY_BURST_COUNT: usize   = 14;

impl Plugin for ParticleSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                setup_cloud_emitters,  // detecta nubes por CloudGoopAnimated
                setup_enemy_emitters,  // detecta enemigos por Enemy component
                cloud_particles,
                enemy_particles,
                dark_particle_tick,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

// -----------------------------------------------
// COMPONENTES
// -----------------------------------------------

#[derive(Component)]
pub struct CloudEmitter {
    timer: f32,
}

#[derive(Component)]
pub struct EnemyEmitter {
    timer: f32,
    burst_done: bool, // evita re-burst mientras knockback sigue activo
}

#[derive(Component)]
pub struct DarkParticle {
    velocity: Vec3,
    lifetime: f32,
    max_lifetime: f32,
}

// -----------------------------------------------
// SETUP — añade emitters a nubes y enemigos
// -----------------------------------------------

/// Detecta nubes por CloudGoopAnimated (ya procesadas por cloud_goop.rs)
fn setup_cloud_emitters(
    mut commands: Commands,
    query: Query<Entity, (Added<CloudGoopAnimated>, Without<CloudEmitter>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(CloudEmitter { timer: 0.0 });
        info!("CloudEmitter añadido a nube");
    }
}

/// Detecta enemigos por componente Enemy — sin depender del nombre
fn setup_enemy_emitters(
    mut commands: Commands,
    query: Query<Entity, (Added<Enemy>, Without<EnemyEmitter>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(EnemyEmitter {
            timer: 0.0,
            burst_done: false,
        });
        info!("EnemyEmitter añadido a enemigo {:?}", entity);
    }
}

// -----------------------------------------------
// PARTÍCULAS DE NUBES — chispas oscuras
// -----------------------------------------------

fn cloud_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut emitters: Query<(&GlobalTransform, &mut CloudEmitter)>,
) {
    let dt = time.delta_secs();
    for (transform, mut emitter) in emitters.iter_mut() {
        emitter.timer -= dt;
        if emitter.timer > 0.0 { continue; }
        emitter.timer = CLOUD_SPAWN_INTERVAL;

        spawn_dark_particles(
            &mut commands,
            &mut meshes,
            &mut materials,
            transform.translation(),
            CLOUD_PARTICLE_COUNT,
            ParticleStyle::Cloud,
        );
    }
}

// -----------------------------------------------
// PARTÍCULAS DE ENEMIGOS — idle + burst al knockback
// -----------------------------------------------

fn enemy_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut emitters: Query<(&GlobalTransform, &mut EnemyEmitter, Option<&Knockback>), With<Enemy>>,
) {
    let dt = time.delta_secs();
    for (transform, mut emitter, knockback) in emitters.iter_mut() {
        let origin = transform.translation();

        // Burst UNA SOLA VEZ al inicio del knockback
        if knockback.is_some() && !emitter.burst_done {
            emitter.burst_done = true;
            spawn_dark_particles(
                &mut commands,
                &mut meshes,
                &mut materials,
                origin,
                ENEMY_BURST_COUNT,
                ParticleStyle::EnemyBurst,
            );
        }
        if knockback.is_none() {
            emitter.burst_done = false;
        }

        // Idle — emana siempre
        emitter.timer -= dt;
        if emitter.timer > 0.0 { continue; }
        emitter.timer = ENEMY_IDLE_INTERVAL;

        spawn_dark_particles(
            &mut commands,
            &mut meshes,
            &mut materials,
            origin,
            ENEMY_IDLE_COUNT,
            ParticleStyle::EnemyIdle,
        );
    }
}

// -----------------------------------------------
// TICK — mueve, aplica gravedad suave y mata partículas
// -----------------------------------------------

fn dark_particle_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut DarkParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime -= dt;
        particle.velocity += Vec3::NEG_Y * 2.5 * dt; // gravedad suave
        transform.translation += particle.velocity * dt;

        // Fade out por escala
        let life_ratio = (particle.lifetime / particle.max_lifetime).max(0.0);
        transform.scale = Vec3::splat(life_ratio * particle.max_lifetime * 0.08);

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// -----------------------------------------------
// HELPER — spawn partículas oscuras
// -----------------------------------------------

enum ParticleStyle {
    Cloud,
    EnemyIdle,
    EnemyBurst,
}

fn spawn_dark_particles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    count: usize,
    style: ParticleStyle,
) {
    let mut rng = rand::rng();

    let (base_color, emissive, speed_min, speed_max, life_min, life_max, spread_y_min, spread_y_max) =
        match style {
            ParticleStyle::Cloud => (
                pick_dark_color(&mut rng),
                EMISSIVE_CLOUD,
                0.8_f32, 2.5, 0.6, 1.5, -0.4_f32, 0.6,
            ),
            ParticleStyle::EnemyIdle => (
                pick_dark_color(&mut rng),
                EMISSIVE_ENEMY,
                0.3_f32, 1.2, 0.4, 0.9, 0.1_f32, 0.9,
            ),
            ParticleStyle::EnemyBurst => (
                COLOR_WINE,
                EMISSIVE_ENEMY,
                2.0_f32, 5.5, 0.5, 1.2, 0.3_f32, 2.2,
            ),
        };

    let mesh = meshes.add(Sphere { radius: 1.0 });
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(base_color.0, base_color.1, base_color.2),
        emissive: LinearRgba::new(emissive.0, emissive.1, emissive.2, 1.0),
        ..default()
    });

    for _ in 0..count {
        let dir = Vec3::new(
            rng.random_range(-1.0..1.0_f32),
            rng.random_range(spread_y_min..spread_y_max),
            rng.random_range(-1.0..1.0_f32),
        )
        .normalize_or_zero();

        let speed    = rng.random_range(speed_min..speed_max);
        let lifetime = rng.random_range(life_min..life_max);
        let scale    = rng.random_range(0.05..0.11_f32);

        commands.spawn((
            Name::new("DarkParticle"),
            DarkParticle {
                velocity: dir * speed,
                lifetime,
                max_lifetime: lifetime,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(scale)),
        ));
    }
}

fn pick_dark_color(rng: &mut impl rand::Rng) -> (f32, f32, f32) {
    match rng.random_range(0..3_u32) {
        0 => COLOR_BLACK,
        1 => COLOR_WINE,
        _ => COLOR_DARK_PURPLE,
    }
}
