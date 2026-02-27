use bevy::prelude::*;
use avian3d::prelude::*;
use crate::screens::Screen;
use crate::screens::gameplay::enemy::EnemySpawnCmd;
use crate::screens::gameplay::Player;

/* ------------------------------------------------ */
/* ---------------- COMPONENTES ------------------- */
/* ------------------------------------------------ */
#[derive(Component)]
pub struct SpawnZone;

#[derive(Component)]
pub struct SpawnZoneActivated;

/* ------------------------------------------------ */
/* ---------------- PLUGIN ------------------------ */
/* ------------------------------------------------ */
pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tag_arena_zones,
                setup_spawn_sensors,
                detect_spawn_trigger,
                spawn_from_zones,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

/* ------------------------------------------------ */
/* -------- DETECTAR ARENAS DESDE BLENDER ---------- */
/* ------------------------------------------------ */
fn tag_arena_zones(
    mut commands: Commands,
    query: Query<(Entity, &Name), Added<Name>>,
) {
    for (entity, name) in &query {
        let name_lower = name.as_str().to_lowercase();
        if name_lower.contains("collision_arena") {
            info!("SpawnZone detectada: {}", name.as_str());
            commands.entity(entity).insert(SpawnZone);
        }
    }
}

/* ------------------------------------------------ */
/* --------- CONVERTIR EN SENSOR ------------------ */
/* ------------------------------------------------ */
fn setup_spawn_sensors(
    mut commands: Commands,
    query: Query<Entity, Added<SpawnZone>>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            Sensor,
            CollisionEventsEnabled,  // ← REQUERIDO para recibir CollisionStart
        ));
    }
}

/* ------------------------------------------------ */
/* ------- DETECTAR ENTRADA DEL JUGADOR ------------ */
/* ------------------------------------------------ */
fn detect_spawn_trigger(
    mut commands: Commands,
    mut collisions: MessageReader<CollisionStart>,  // ← MessageReader, no EventReader
    zones: Query<Entity, With<SpawnZone>>,
    activated: Query<Entity, With<SpawnZoneActivated>>,
    players: Query<Entity, With<Player>>,
) {
    for ev in collisions.read() {
        let a = ev.collider1;  // CollisionStart tiene campos nombrados
        let b = ev.collider2;

        let zone_player =
            (zones.get(a).is_ok() && players.get(b).is_ok())
                || (zones.get(b).is_ok() && players.get(a).is_ok());

        if !zone_player {
            continue;
        }

        let zone = if zones.get(a).is_ok() { a } else { b };

        if activated.get(zone).is_ok() {
            continue;
        }

        info!("Zona activada: {:?}", zone);
        commands.entity(zone).insert(SpawnZoneActivated);
    }
}

/* ------------------------------------------------ */
/* --------- HACER SPAWN REAL ---------------------- */
/* ------------------------------------------------ */
fn spawn_from_zones(
    mut commands: Commands,
    zones: Query<(Entity, &Transform), Added<SpawnZoneActivated>>,
) {
    for (entity, tf) in &zones {
        info!("Spawneando enemigos en {:?}", entity);
        spawn_enemies(commands.reborrow(), tf.translation);
    }
}

/* ------------------------------------------------ */
/* --------- PATRÓN DE SPAWN ----------------------- */
/* ------------------------------------------------ */
fn spawn_enemies(
    mut commands: Commands,
    center: Vec3,
) {
    let height = 12.0;
    let positions = [
        center + Vec3::new(3.0, height, 0.0),
        center + Vec3::new(-3.0, height, 2.0),
        center + Vec3::new(0.0, height + 2.0, -3.0),
    ];
    for pos in positions {
        commands.queue(EnemySpawnCmd {
            transform: Transform::from_translation(pos),
            parent: None,
        });
    }
}
