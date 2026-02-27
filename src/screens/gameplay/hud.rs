use bevy::prelude::*;
use crate::screens::Screen;
use super::player::Player;
use super::enemy::Enemy;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KillStreak::default());
        app.insert_resource(CirclesSpawned::default());
        app.add_systems(OnEnter(Screen::Gameplay), spawn_overlays);
        app.add_systems(
            Update,
            (
                update_damage_overlay,
                track_kills,
                decay_streak,
                update_hallucination_overlay,
                spawn_hallucination_circles,
                update_hallucination_circles,
            )
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

// -----------------------------------------------
// RECURSOS
// -----------------------------------------------

#[derive(Resource, Default)]
pub struct KillStreak {
    pub kills: u32,
    pub decay_timer: f32,
}

#[derive(Resource, Default)]
struct CirclesSpawned(bool);

// -----------------------------------------------
// COMPONENTES
// -----------------------------------------------

#[derive(Component)]
struct DamageOverlay;

#[derive(Component)]
struct HallucinationOverlay;

#[derive(Component)]
struct HallucinationCircle {
    speed_x: f32,
    speed_y: f32,
    phase_x: f32,
    phase_y: f32,
    base_x: f32,
    base_y: f32,
    size: f32,
}

#[derive(Bundle)]
struct HalCircleBundle {
    name: Name,
    circle: HallucinationCircle,
    node: Node,
    background: BackgroundColor,
    z_index: ZIndex,
    global_z: GlobalZIndex,
}

// -----------------------------------------------
// SPAWN
// -----------------------------------------------

fn spawn_overlays(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Overlay de daño (sangre)
    commands.spawn((
        Name::new("DamageOverlay"),
        DamageOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ImageNode {
            image: asset_server.load("images/player_damage.png"),
            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            ..default()
        },
        ZIndex(15),
        GlobalZIndex(15),
    ));

    // Overlay naranja suave de racha — máximo 30% alpha
    commands.spawn((
        Name::new("HallucinationOverlay"),
        HallucinationOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ImageNode {
                image: asset_server.load("images/vignette_hallucination.png"),
                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                ..default()
            },
        ZIndex(12),
        GlobalZIndex(12),
    ));
}

fn spawn_hallucination_circles(
    mut commands: Commands,
    mut spawned: ResMut<CirclesSpawned>,
    streak: Res<KillStreak>,
) {
    if spawned.0 || streak.kills == 0 {
        return;
    }
    spawned.0 = true;

    let circle_data: [(f32, f32, f32, f32); 12] = [
        (15.0, 20.0, 0.3, 80.0),
        (70.0, 60.0, 1.1, 120.0),
        (40.0, 80.0, 2.3, 90.0),
        (85.0, 15.0, 0.7, 150.0),
        (25.0, 50.0, 1.8, 100.0),
        (60.0, 35.0, 3.1, 110.0),
        (10.0, 70.0, 0.5, 130.0),
        (75.0, 85.0, 2.7, 95.0),
        (50.0, 10.0, 1.4, 140.0),
        (30.0, 40.0, 0.9, 105.0),
        (90.0, 55.0, 2.0, 115.0),
        (45.0, 90.0, 1.6, 125.0),
    ];

    for (i, (bx, by, phase, size)) in circle_data.iter().enumerate() {
        commands.spawn(HalCircleBundle {
            name: Name::new(format!("HalCircle_{}", i)),
            circle: HallucinationCircle {
                speed_x: 0.4 + i as f32 * 0.15,
                speed_y: 0.3 + i as f32 * 0.12,
                phase_x: *phase,
                phase_y: *phase + 1.0,
                base_x: *bx,
                base_y: *by,
                size: *size,
            },
            node: Node {
                position_type: PositionType::Absolute,
                width: Val::Px(*size),
                height: Val::Px(*size),
                left: Val::Percent(*bx),
                top: Val::Percent(*by),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            background: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
            z_index: ZIndex(13),
            global_z: GlobalZIndex(13),
        });
    }
}

// -----------------------------------------------
// SISTEMAS
// -----------------------------------------------

fn update_damage_overlay(
    player: Single<&Player>,
    mut overlay: Single<&mut ImageNode, With<DamageOverlay>>,
) {
    let damage_amount = 1.0 - player.health;
    overlay.color = Color::srgba(1.0, 1.0, 1.0, damage_amount * 0.85);
}

fn update_hallucination_overlay(
    streak: Res<KillStreak>,
    time: Res<Time>,
    mut overlay: Single<&mut ImageNode, With<HallucinationOverlay>>,
) {
    let intensity = (streak.kills as f32 / 5.0).clamp(0.0, 1.0);
    let pulse = (time.elapsed_secs() * (1.0 + intensity * 2.0)).sin() * 0.04 * intensity;
    let alpha = (intensity * 0.26 + pulse).max(0.0);
    overlay.color = Color::srgba(1.0, 1.0, 1.0, alpha);
}

fn track_kills(
    enemies: Query<(), With<Enemy>>,
    mut enemy_count: Local<u32>,
    mut streak: ResMut<KillStreak>,
) {
    let current_count = enemies.iter().count() as u32;
    if current_count < *enemy_count {
        let kills_this_frame = *enemy_count - current_count;
        streak.kills += kills_this_frame;
        streak.decay_timer = 6.0;
    }
    *enemy_count = current_count;
}

fn decay_streak(
    mut streak: ResMut<KillStreak>,
    time: Res<Time>,
) {
    if streak.decay_timer > 0.0 {
        streak.decay_timer -= time.delta_secs();
        if streak.decay_timer <= 0.0 {
            if streak.kills > 0 {
                streak.kills -= 1;
                streak.decay_timer = if streak.kills > 0 { 2.0 } else { 0.0 };
            }
        }
    }
}

fn update_hallucination_circles(
    streak: Res<KillStreak>,
    time: Res<Time>,
    mut circles: Query<(&HallucinationCircle, &mut Node, &mut BackgroundColor)>,
    mut spawned: ResMut<CirclesSpawned>,
) {
    if streak.kills == 0 {
        spawned.0 = false;
        for (_, _, mut color) in circles.iter_mut() {
            color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
        return;
    }

    let intensity = (streak.kills as f32 / 5.0).clamp(0.0, 1.0);
    let chaos = 1.0 + intensity * 4.0;
    let t = time.elapsed_secs();

    for (circle, mut node, mut color) in circles.iter_mut() {
        let offset_x = (t * circle.speed_x * chaos + circle.phase_x).sin() * 15.0 * chaos;
        let offset_y = (t * circle.speed_y * chaos + circle.phase_y).cos() * 12.0 * chaos;

        node.left = Val::Percent(circle.base_x + offset_x);
        node.top = Val::Percent(circle.base_y + offset_y);

        let pulse = (t * 2.0 * chaos + circle.phase_x).sin() * 0.3 + 1.0;
        let current_size = circle.size * pulse * (0.3 + intensity * 0.7);
        node.width = Val::Px(current_size);
        node.height = Val::Px(current_size);

        // Círculos blancos con alpha que sube con el streak
        let alpha = intensity * 0.6 * ((t * chaos + circle.phase_y).sin() * 0.3 + 0.7);
        color.0 = Color::srgba(1.0, 1.0, 1.0, alpha.max(0.0));
    }
}
