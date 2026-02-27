use bevy::prelude::*;

use crate::screens::gameplay::enemy_spawn::EnemySpawn;
use crate::screens::gameplay::enemy::EnemySpawnCmd;
use crate::screens::gameplay::enemy_spawn::SpawnConsumed;

use crate::screens::Screen;
use crate::screens::gameplay::{
    enemy::spawn_enemy,
    LevelAssets,
    NavmeshArchipelagoHolder,
};

/// ===============================
/// RESOURCE: CONTROL DE OLEADAS
/// ===============================

#[derive(Resource)]
pub struct WaveManager {
    pub timer: Timer,
    pub current_wave: i32,
    pub current_zone: String,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            current_wave: 0,
            current_zone: "first_arena".to_string(),
        }
    }
}

/// ===============================
/// COMPONENTE: SPAWN DESDE BLENDER
/// (Importado por bevy_skein)
/// ===============================


/// Marca que ya fue usado
// #[derive(Component)]
// pub struct SpawnConsumed;

/// ===============================
/// PLUGIN
/// ===============================

pub struct WaveSpawnPlugin;

impl Plugin for WaveSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemySpawn>();

        app.init_resource::<WaveManager>();

        app.add_systems(
            OnEnter(Screen::Gameplay),
            setup_waves,
        );

        app.add_systems(
            Update,
            update_waves
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

/// ===============================
/// SETUP
/// ===============================

fn setup_waves(
    mut manager: ResMut<WaveManager>,
) {
    manager.current_wave = 0;
    manager.timer.reset();

    info!("🌊 Wave system initialized");
}

/// ===============================
/// SISTEMA PRINCIPAL
/// ===============================

fn update_waves(
    time: Res<Time>,
    mut manager: ResMut<WaveManager>,
    mut commands: Commands,
    // Eliminamos Without<SpawnConsumed> para que los puntos sean reutilizables
    spawns: Query<(&Name, &GlobalTransform, Option<&EnemySpawn>)>,
) {
    manager.timer.tick(time.delta());

    if !manager.timer.just_finished() {
        return;
    }

    manager.current_wave += 1;
    info!("🌊 Wave Check: Ola {}, Zona {}", manager.current_wave, manager.current_zone);

    let mut count = 0;
    for (name, transform, maybe_spawn) in &spawns {
        // 1. Filtro por nombre (visto en Blender)
        if !name.as_str().contains("EnemySpawn") {
            continue;
        }

        // 2. Filtro por componente (si Skein lo carga correctamente)
        // Si el componente existe, validamos zona. Si no existe, spawneamos por defecto.
        if let Some(spawn) = maybe_spawn {
            if spawn.zone != manager.current_zone {
                continue;
            }
            // Si quieres que un Empty específico solo aparezca en una ola concreta:
             if spawn.wave != manager.current_wave { continue; }
        }

        let pos = transform.translation();

        // Evitamos el origen por errores de carga de escena
        if pos == Vec3::ZERO { continue; }

        info!("👾 Ola {}: Spawning enemigo desde '{}' en {:?}", manager.current_wave, name, pos);

        // Ejecutamos el comando de spawn
        commands.queue(EnemySpawnCmd {
            transform: Transform::from_translation(pos),
            parent: None,
        });

        count += 1;
    }

    if count > 0 {
        info!("🚀 Ola {} completada: {} enemigos generados", manager.current_wave, count);
    } else {
        warn!("⚠️ No se encontraron puntos de spawn válidos para la zona: {}", manager.current_zone);
    }
}
