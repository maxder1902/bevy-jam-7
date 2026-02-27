//! Sistema del reloj de alarma — arma especial que paraliza enemigos
//!
//! Flujo simplificado:
//! Enemy muerte → drop reloj → jugador recoge (F) → jugador lanza (RMB) →
//! vuela 0.5s → time field aparece y crece → enemigos paralizados → fade

use bevy::prelude::*;
use avian3d::prelude::*;
use rand::RngExt;
use crate::screens::Screen;
use crate::screens::gameplay::LevelAssets;
use crate::screens::gameplay::enemy::Enemy;
use crate::screens::gameplay::events::SpawnAlarmClockEvent;
pub struct AlarmClockPlugin;

const PICKUP_RANGE: f32 = 2.5;
const THROW_SPEED: f32 = 18.0;
const TIME_FIELD_RADIUS: f32 = 10.0;
const TIME_FIELD_DURATION: f32 = 5.0;
const FREEZE_DURATION: f32 = 5.0;
const STAR_COUNT: usize = 12;

impl Plugin for AlarmClockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_picks_alarm_clock,
                player_throw_alarm_clock,
                alarm_clock_tick,
            ).run_if(in_state(Screen::Gameplay)),
        );
        app.add_systems(
            Update,
            (
                time_field_tick,
                frozen_enemy_tick,
                star_tick,
            ).run_if(in_state(Screen::Gameplay)),
        );
    }
}

// -----------------------------------------------
// COMPONENTES
// -----------------------------------------------

#[derive(Component)]
pub struct AlarmClock {
    pub thrown_timer: f32,
    pub is_thrown: bool,
}

#[derive(Component)]
pub struct HeldClock;

#[derive(Component)]
pub struct TimeField {
    pub timer: f32,
}

#[derive(Component)]
pub struct FrozenEnemy {
    pub remaining: f32,
}

#[derive(Component)]
pub struct ClockStar {
    pub velocity: Vec3,
    pub lifetime: f32,
}

// -----------------------------------------------
// SPAWN DEL RELOJ
// -----------------------------------------------

pub fn spawn_alarm_clock(
    commands: &mut Commands,
    level_assets: &LevelAssets,
    position: Vec3,
) {
    commands.spawn((
        Name::new("AlarmClock"),
        AlarmClock { thrown_timer: 0.0, is_thrown: false },
        SceneRoot(level_assets.alarm_clock_scene.clone()),
        Transform::from_translation(position),
        RigidBody::Dynamic,
        Collider::sphere(0.3),
        LinearVelocity::default(),
        AngularVelocity::default(),
    ));
}

// -----------------------------------------------
// RECOGER
// -----------------------------------------------

fn player_picks_alarm_clock(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Single<&Transform, With<super::player::Player>>,
    camera: Single<Entity, With<Camera3d>>,
    mut clocks: Query<(Entity, &Transform, &mut AlarmClock), Without<HeldClock>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) { return; }

    let player_pos = player.translation;

    for (entity, clock_transform, mut clock) in clocks.iter_mut() {
        if clock.is_thrown { continue; }

        let dist = player_pos.distance(clock_transform.translation);
        if dist > PICKUP_RANGE { continue; }

        commands.entity(entity)
            .remove::<RigidBody>()
            .remove::<Collider>()
            .remove::<LinearVelocity>()
            .remove::<AngularVelocity>()
            .insert((
                HeldClock,
                ChildOf(*camera),
                Transform::from_translation(Vec3::new(0.3, -0.4, -0.8))
                    .with_rotation(Quat::from_rotation_y(0.3))
                    .with_scale(Vec3::splat(0.6)),
            ));

        info!("Reloj recogido!");
        break;
    }
}

// -----------------------------------------------
// LANZAR
// -----------------------------------------------

fn player_throw_alarm_clock(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Transform, &GlobalTransform), With<Camera3d>>,
    mut clocks: Query<(Entity, &mut AlarmClock), With<HeldClock>>,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }

    let (_, cam_global) = *camera;

    for (entity, mut clock) in clocks.iter_mut() {
        let forward = cam_global.forward();
        let throw_velocity = forward * THROW_SPEED + Vec3::Y * 3.0;
        let world_pos = cam_global.translation() + forward * 1.0;

        commands.entity(entity)
            .remove::<HeldClock>()
            .remove::<ChildOf>()
            .insert((
                Transform::from_translation(world_pos),
                RigidBody::Dynamic,
                Collider::sphere(0.3),
                LinearVelocity(throw_velocity),
                AngularVelocity(Vec3::new(5.0, 3.0, 2.0)),
            ));

        clock.is_thrown = true;
        clock.thrown_timer = 0.0;
        info!("Reloj lanzado!");
        break;
    }
}

// -----------------------------------------------
// TICK DEL RELOJ
// -----------------------------------------------

fn alarm_clock_tick(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut clocks: Query<(Entity, &Transform, &mut AlarmClock)>,  // ← quita LinearVelocity
) {
    for (entity, transform, mut clock) in clocks.iter_mut() {
        if !clock.is_thrown { continue; }

        clock.thrown_timer += time.delta_secs();

        // Dispara a los 0.8s sin importar velocidad ni dirección
        if clock.thrown_timer < 0.8 { continue; }

        let pos = transform.translation;
        info!("Time field spawneado en {:?}", pos);

        commands.spawn((
            Name::new("TimeField"),
            TimeField { timer: TIME_FIELD_DURATION },
            SceneRoot(level_assets.time_field_scene.clone()),
            Transform::from_translation(pos).with_scale(Vec3::ZERO),
        ));

        spawn_stars(&mut commands, &mut meshes, &mut materials, pos);
        commands.entity(entity).despawn();
    }
}
// -----------------------------------------------
// ESTRELLITAS
// -----------------------------------------------

fn spawn_stars(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
) {
    let mut rng = rand::rng();
    let star_mesh = meshes.add(Sphere { radius: 0.08 });
    let star_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.3),
        emissive: LinearRgba::new(3.0, 2.5, 0.5, 1.0),
        ..default()
    });

    for _ in 0..STAR_COUNT {
        let dir = Vec3::new(
            rng.random_range(-1.0..1.0_f32),
            rng.random_range(0.5..1.5_f32),
            rng.random_range(-1.0..1.0_f32),
        ).normalize_or_zero();

        let speed = rng.random_range(3.0..7.0_f32);

        commands.spawn((
            Name::new("ClockStar"),
            ClockStar {
                velocity: dir * speed,
                lifetime: rng.random_range(0.8..1.5_f32),
            },
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_mat.clone()),
            Transform::from_translation(origin),
        ));
    }
}

fn star_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut stars: Query<(Entity, &mut Transform, &mut ClockStar)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut star) in stars.iter_mut() {
        star.lifetime -= dt;
        star.velocity += Vec3::NEG_Y * 9.81 * dt;
        transform.translation += star.velocity * dt;
        if star.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// -----------------------------------------------
// TIME FIELD
// -----------------------------------------------

fn time_field_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut fields: Query<(Entity, &mut Transform, &mut TimeField)>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<FrozenEnemy>, Without<TimeField>)>,
) {
    for (field_entity, mut field_transform, mut field) in fields.iter_mut() {
        field.timer -= time.delta_secs();

        let elapsed = TIME_FIELD_DURATION - field.timer;
        let scale = (elapsed / 0.3).min(1.0) * 5.0;
        field_transform.scale = Vec3::splat(scale);

        let field_pos = field_transform.translation;
        for (enemy_entity, enemy_transform) in enemies.iter() {
            if enemy_transform.translation.distance(field_pos) <= TIME_FIELD_RADIUS {
                commands.entity(enemy_entity).insert(FrozenEnemy {
                    remaining: FREEZE_DURATION,
                });
                info!("Enemigo paralizado!");
            }
        }

        if field.timer <= 0.0 {
            commands.entity(field_entity).despawn();
            info!("Time field expirado");
        }
    }
}

// -----------------------------------------------
// ENEMIGOS CONGELADOS
// -----------------------------------------------

fn frozen_enemy_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut frozen: Query<(Entity, &mut FrozenEnemy, &mut LinearVelocity)>,
) {
    for (entity, mut frozen, mut velocity) in frozen.iter_mut() {
        velocity.x = 0.0;
        velocity.z = 0.0;

        frozen.remaining -= time.delta_secs();
        if frozen.remaining <= 0.0 {
            commands.entity(entity).remove::<FrozenEnemy>();
            info!("Enemigo descongelado");
        }
    }
}
