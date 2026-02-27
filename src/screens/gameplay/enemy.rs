use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use crate::screens::Screen;
use crate::screens::gameplay::LevelAssets;
use crate::screens::gameplay::hammerhead::HammerheadAssets;
use crate::screens::gameplay::alarm_clock::FrozenEnemy;

pub struct EnemyPlugin;

const ENEMY_GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);
const MAX_HEALTH: u32 = 3;
const ENEMY_SPEED: f32 = 3.0;
const DETECTION_RANGE: f32 = 20.0;
const ATTACK_RANGE: f32 = 2.2;
const ATTACK_DAMAGE: f32 = 0.25;
const ATTACK_COOLDOWN: f32 = 5.0;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct EnemyHealthBillboard {
    pub enemy: Entity,
}

static ENEMY_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                setup_enemy_animations,
                enemy_chase_and_attack,
                apply_knockback,
                update_grounded,
                apply_gravity,
                enemy_health_system,
                sync_billboard_position,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );

        app.add_systems(
            PhysicsSchedule,
            enemy_collision.in_set(NarrowPhaseSystems::Last),
        );
    }
}

#[derive(Component)]
pub struct Enemy {
    pub id: u32,
    pub health: u32,
    pub attack_cooldown: f32,
}

#[derive(Component)]
pub struct EnemyAnimationPlayer {
    pub enemy: Entity,
}

#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec3,
    pub remaining_time: f32,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

pub struct EnemySpawnCmd {
    pub transform: Transform,
    pub parent: Option<Entity>,
}

impl Command for EnemySpawnCmd {
    fn apply(self, world: &mut World) {
        world.run_system_cached_with(spawn_enemy, self).unwrap();
    }
}

pub fn spawn_enemy(
    In(args): In<EnemySpawnCmd>,
    mut c: Commands,
    level_assets: Res<LevelAssets>,
) {
    let enemy_id = ENEMY_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    let enemy_collider = Collider::capsule(0.45, 1.3);
    let mut caster_shape = enemy_collider.clone();
    caster_shape.set_scale(Vec3::ONE * 0.99, 10);

    let enemy_entity = c.spawn((
        Name::new(format!("Enemy_{}", enemy_id)),
        Enemy {
            id: enemy_id,
            health: MAX_HEALTH,
            attack_cooldown: 0.0,
        },
        SceneRoot(level_assets.hammerhead.scene.clone()),
        args.transform,
        Visibility::Inherited,
        RigidBody::Kinematic,
        LinearVelocity::default(),
        ShapeCaster::new(
            caster_shape,
            Vec3::new(0.0, 1.17, 0.0),
            Quaternion::default(),
            Dir3::NEG_Y,
        )
        .with_max_distance(0.5),
    ))
    .with_children(|parent| {
        parent.spawn((
            Collider::capsule(0.45, 1.3),
            Transform::from_xyz(0.0, 1.17, 0.0),
        ));
    })
    .id();

    c.spawn((
        Name::new(format!("HealthBillboard_{}", enemy_id)),
        EnemyHealthBillboard { enemy: enemy_entity },
        HealthText,
        Text::new("XXX"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(bevy::color::palettes::css::RED.into()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        ZIndex(10),
    ));
}

// -----------------------------------------------
// ANIMACIONES
// -----------------------------------------------

fn setup_enemy_animations(
    mut commands: Commands,
    new_players: Query<(Entity, &ChildOf), (Added<AnimationPlayer>, Without<EnemyAnimationPlayer>)>,
    enemies: Query<Entity, With<Enemy>>,
    level_assets: Res<LevelAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (anim_entity, child_of) in new_players.iter() {
        if enemies.get(child_of.0).is_ok() {
            let enemy_entity = child_of.0;

            let (graph, _node_indices) = AnimationGraph::from_clips(
                level_assets.hammerhead.animations.clone()
            );
            let graph_handle = graphs.add(graph);

            commands.entity(anim_entity).insert((
                EnemyAnimationPlayer { enemy: enemy_entity },
                AnimationGraphHandle(graph_handle),
            ));

            info!("Enemy animation player linked to Enemy {:?}", enemy_entity);
        }
    }
}

fn enemy_chase_and_attack(
    mut enemies: Query<
        (Entity, &Transform, &mut LinearVelocity, &mut Rotation, &mut Enemy),
        Without<FrozenEnemy>,
    >,
    mut players: Query<(&Transform, &mut super::player::Player), With<super::Player>>,
    mut anim_players: Query<(&EnemyAnimationPlayer, &mut AnimationPlayer, &mut AnimationTransitions)>,
    _level_assets: Res<LevelAssets>,
    _graphs: ResMut<Assets<AnimationGraph>>,
    time: Res<Time>,
) {
    const ANIM_ATTACK: usize = 0;
    const ANIM_IDLE: usize = 1;
    const ANIM_RUN: usize = 2;

    let Ok((player_transform, mut player)) = players.single_mut() else { return; };
    let player_pos = player_transform.translation;

    for (enemy_entity, enemy_transform, mut linear_velocity, mut rotation, mut enemy) in enemies.iter_mut() {
        let enemy_pos = enemy_transform.translation;
        let dist = enemy_pos.distance(player_pos);

        enemy.attack_cooldown -= time.delta_secs();

        let anim = anim_players.iter_mut()
            .find(|(ap, _, _)| ap.enemy == enemy_entity);

        if dist > DETECTION_RANGE {
            linear_velocity.x = 0.0;
            linear_velocity.z = 0.0;
            if let Some((_, mut player, mut transitions)) = anim {
                if !player.is_playing_animation(AnimationNodeIndex::new(ANIM_IDLE + 1)) {
                    transitions.play(&mut player, AnimationNodeIndex::new(ANIM_IDLE + 1), Duration::from_millis(300)).repeat();
                }
            }
            continue;
        }

        if dist <= ATTACK_RANGE {
            linear_velocity.x = 0.0;
            linear_velocity.z = 0.0;

            if enemy.attack_cooldown <= 0.0 {
                enemy.attack_cooldown = ATTACK_COOLDOWN;
                player.health = (player.health - ATTACK_DAMAGE).max(0.0);
                info!("Enemy atacó al jugador! Player health: {:.2}", player.health);

                if let Some((_, mut anim_player, mut transitions)) = anim {
                    transitions.play(&mut anim_player, AnimationNodeIndex::new(ANIM_ATTACK + 1), Duration::from_millis(100));
                }
            } else {
                if let Some((_, mut anim_player, mut transitions)) = anim {
                    if !anim_player.is_playing_animation(AnimationNodeIndex::new(ANIM_IDLE + 1)) {
                        transitions.play(&mut anim_player, AnimationNodeIndex::new(ANIM_IDLE + 1), Duration::from_millis(300)).repeat();
                    }
                }
            }
        } else {
            let dir = Vec3::new(player_pos.x - enemy_pos.x, 0.0, player_pos.z - enemy_pos.z).normalize_or_zero();
            linear_velocity.x = dir.x * ENEMY_SPEED;
            linear_velocity.z = dir.z * ENEMY_SPEED;

            if dir.length_squared() > 0.001 {
                *rotation = Quat::from_rotation_y((-dir.x).atan2(-dir.z)).into();
            }

            if let Some((_, mut anim_player, mut transitions)) = anim {
                if !anim_player.is_playing_animation(AnimationNodeIndex::new(ANIM_RUN + 1)) {
                    transitions.play(&mut anim_player, AnimationNodeIndex::new(ANIM_RUN + 1), Duration::from_millis(200)).repeat();
                }
            }
        }
    }
}

// -----------------------------------------------
// FÍSICA
// -----------------------------------------------

fn apply_knockback(
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut LinearVelocity, &mut Knockback)>,
    time: Res<Time>,
) {
    for (entity, mut linear_velocity, mut knockback) in enemies.iter_mut() {
        knockback.velocity += ENEMY_GRAVITY * time.delta_secs();
        linear_velocity.0 = knockback.velocity;
        knockback.remaining_time -= time.delta_secs();
        if knockback.remaining_time <= 0.0 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &ShapeHits, &Rotation), (With<Enemy>, Without<Knockback>)>,
) {
    for (entity, hits, rotation) in &mut query {
        let is_grounded = hits.iter().any(|hit| {
            (rotation * -hit.normal2).angle_between(Vector::Y).abs() <= 35f32.to_radians()
        });
        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut enemies: Query<&mut LinearVelocity, (With<Enemy>, Without<Knockback>, Without<Grounded>)>,
) {
    for mut linear_velocity in enemies.iter_mut() {
        linear_velocity.0 += ENEMY_GRAVITY * time.delta_secs();
    }
}

fn enemy_health_system(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    enemies: Query<(Entity, &Enemy, &Transform)>,
    billboards: Query<(Entity, &EnemyHealthBillboard)>,
) {
    for (entity, enemy, transform) in enemies.iter() {
        if enemy.health == 0 {
            for (billboard_entity, billboard) in billboards.iter() {
                if billboard.enemy == entity {
                    commands.entity(billboard_entity).despawn();
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

fn sync_billboard_position(
    enemies: Query<&Transform, With<Enemy>>,
    mut billboards: Query<(&EnemyHealthBillboard, &mut Node)>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((camera, cam_global)) = camera.single() else { return; };
    for (billboard, mut node) in billboards.iter_mut() {
        if let Ok(enemy_transform) = enemies.get(billboard.enemy) {
            if enemy_transform.translation.y < -50.0 { continue; }
            let world_pos = enemy_transform.translation + Vec3::new(0.0, 3.2, 0.0);
            if let Ok(screen_pos) = camera.world_to_viewport(cam_global, world_pos) {
                node.left = Val::Px(screen_pos.x - 20.0);
                node.top = Val::Px(screen_pos.y - 20.0);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn enemy_collision(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut enemies: Query<(&mut Position, &mut LinearVelocity), With<Enemy>>,
    time: Res<Time>,
) {
    let max_slope_angle = Some(0.1f32);
    for contacts in collisions.iter() {
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else { continue; };

        let is_first: bool;
        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity) = if let Ok(enemy) = enemies.get_mut(rb1) {
            is_first = true;
            character_rb = *bodies.get(rb1).unwrap();
            is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
            enemy
        } else if let Ok(character) = enemies.get_mut(rb2) {
            is_first = false;
            character_rb = *bodies.get(rb2).unwrap();
            is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
            character
        } else { continue; };

        if !character_rb.is_kinematic() { continue; }

        for manifold in contacts.manifolds.iter() {
            let normal = if is_first { -manifold.normal } else { manifold.normal };
            let mut deepest_penetration: Scalar = Scalar::MIN;

            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            if is_other_dynamic { continue; }

            let slope_angle = normal.angle_between(Vector::Y);
            let climbable = max_slope_angle.is_some_and(|angle| slope_angle.abs() <= angle);

            if deepest_penetration > 0.0 {
                if climbable {
                    let normal_direction_xz = normal.reject_from_normalized(Vector::Y).normalize_or_zero();
                    let linear_velocity_xz = linear_velocity.dot(normal_direction_xz);
                    let max_y_speed = -linear_velocity_xz * slope_angle.tan();
                    linear_velocity.y = linear_velocity.y.max(max_y_speed);
                } else {
                    if linear_velocity.dot(normal) > 0.0 { continue; }
                    let impulse = linear_velocity.reject_from_normalized(normal);
                    linear_velocity.0 = impulse;
                }
            } else {
                let normal_speed = linear_velocity.dot(normal);
                if normal_speed > 0.0 { continue; }
                let impulse_magnitude = normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                let mut impulse = impulse_magnitude * normal;
                if climbable {
                    linear_velocity.y -= impulse.y.min(0.0);
                } else {
                    impulse.y = impulse.y.max(0.0);
                    linear_velocity.0 -= impulse;
                }
            }
        }
    }
}
