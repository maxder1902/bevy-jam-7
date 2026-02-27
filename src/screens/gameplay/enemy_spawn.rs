use bevy::prelude::*;
use avian3d::prelude::*;
use crate::screens::gameplay::enemy::EnemySpawnCmd;

#[derive(Component)]
pub struct SpawnConsumed;

pub struct EnemySpawnPlugin;

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemySpawn>();
        // app.add_systems(Update, spawn_enemies_from_scene);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct EnemySpawn {
    pub parent: Option<Entity>,

    // Custom props de Blender
    pub wave: i32,
    pub zone: String,
    pub r#type: String,
    pub delay: f32,
}


fn spawn_enemies_from_scene(
    mut commands: Commands,
    // CAMBIO 1: Usamos ChildOf en lugar de Parent (Bevy 0.15+)
    // CAMBIO 2: Añadimos filtros para asegurar que GlobalTransform y Name estén listos
    nodes: Query<(Entity, &GlobalTransform, &Name), (Without<SpawnConsumed>, With<ChildOf>)>,
) {
    for (entity, g_transform, name) in &nodes {
        if name.as_str().contains("EnemySpawn") {
            // CAMBIO 3: Especificamos el tipo Vec3 para ayudar a la inferencia
            let spawn_pos: Vec3 = g_transform.translation();

            // Evitamos spawnear si la escena aún no se ha posicionado en el mundo
            if spawn_pos == Vec3::ZERO {
                continue;
            }

            info!("Spawning enemy from node: {} at {}", name, spawn_pos);

            commands.queue(EnemySpawnCmd {
                transform: Transform::from_translation(spawn_pos),
                parent: None,
            });

            // Marcamos para no procesar de nuevo en el siguiente frame
            commands.entity(entity).insert(SpawnConsumed);

            // Opcional: Si no necesitas el objeto vacío de Blender para nada más,
            // puedes borrarlo para limpiar la jerarquía:
            // commands.entity(entity).despawn_recursive();
        }
    }
}
