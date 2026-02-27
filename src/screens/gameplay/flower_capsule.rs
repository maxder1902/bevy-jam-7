use bevy::prelude::*;
use crate::screens::Screen;
use avian3d::prelude::*;
use crate::screens::gameplay::LevelAssets;
use crate::audio::sound_effect;
use crate::screens::gameplay::Menu;

pub struct FlowerCapsulePlugin;

#[cfg(feature = "dev")]
const CAPSULE_MAX_HEALTH: u32 = 3;
#[cfg(not(feature = "dev"))]
const CAPSULE_MAX_HEALTH: u32 = 15;

impl Plugin for FlowerCapsulePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CorruptedGlass>();
        app.insert_resource(CapsuleTracker::default());
        app.add_systems(
            Update,
            (
                register_capsules,   // 1. registra cápsulas (Added<Name>)
                register_shards,     // 2. vincula shards (sin Added, espera cápsulas)
                force_capsule_base_color,
                apply_capsule_damage_color,
                flower_capsule_tracking,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

#[derive(Resource, Default)]
pub struct CapsuleTracker {
    pub total: u32,
    pub broken: u32,
}

#[derive(Component)]
pub struct GlassShard;

#[derive(Component)]
pub struct ShardOwner(pub Entity);

impl CapsuleTracker {
    pub fn all_broken(&self) -> bool {
        self.total > 0 && self.broken >= self.total
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CorruptedGlass;

#[derive(Component)]
pub struct FlowerCapsule {
    pub health: u32,
}

impl Default for FlowerCapsule {
    fn default() -> Self {
        Self { health: CAPSULE_MAX_HEALTH }
    }
}

/// Sistema 1 — registra cápsulas en cuanto aparecen (Added<Name>)
fn register_capsules(
    mut commands: Commands,
    mut tracker: ResMut<CapsuleTracker>,
    query: Query<(Entity, &Name), (Added<Name>, Without<FlowerCapsule>)>,
) {
    for (entity, name) in query.iter() {
        if name.as_str().to_lowercase().contains("collision_corrupted_glass") {
            commands.entity(entity).insert(FlowerCapsule::default());
            tracker.total += 1;
            info!("Cápsula registrada: {} (total: {})", name, tracker.total);
        }
    }
}

/// Sistema 2 — vincula shards a su cápsula más cercana.
/// Corre cada frame SIN Added — cuando encuentra un shard sin GlassShard
/// lo registra. El Without<GlassShard> garantiza que no se reprocese.
/// El frame de delay del .chain() garantiza que las cápsulas ya existen.
fn register_shards(
    mut commands: Commands,
    capsule_positions: Query<(Entity, &GlobalTransform), With<FlowerCapsule>>,
    query: Query<
        (Entity, &Name, &GlobalTransform),
        (Without<GlassShard>, Without<FlowerCapsule>),
    >,
) {
    if capsule_positions.is_empty() {
        return;
    }

    for (entity, name, transform) in query.iter() {
        if !name.as_str().to_lowercase().contains("corrupted_glass_shard") {
            continue;
        }

        let shard_pos = transform.translation();

        let owner = capsule_positions
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.translation()
                    .distance(shard_pos)
                    .partial_cmp(&b.translation().distance(shard_pos))
                    .unwrap()
            })
            .map(|(e, _)| e);

        commands.entity(entity).insert((GlassShard, RigidBody::Static));

        if let Some(owner_entity) = owner {
            commands.entity(entity).insert(ShardOwner(owner_entity));
            info!("Shard vinculado: {}", name);
        } else {
            info!("Shard sin cápsula: {}", name);
        }
    }
}

/// Se ejecuta UNA SOLA VEZ cuando el material del GLTF llega — fuerza el color morado
fn force_capsule_base_color(
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Name, &MeshMaterial3d<StandardMaterial>), Added<MeshMaterial3d<StandardMaterial>>>,
) {
    for (name, mat_handle) in query.iter() {
        if name.as_str().to_lowercase().contains("corrupted_glass")
            && !name.as_str().to_lowercase().contains("collision_")
        {
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                mat.base_color = Color::srgb(0.3, 0.0, 0.4);
                mat.perceptual_roughness = 1.0;
                mat.metallic = 0.0;
                mat.specular_transmission = 0.0;
                info!("Color morado forzado en: {}", name);
            }
        }
    }
}

/// Se ejecuta solo cuando FlowerCapsule cambia (Changed) — transiciona a rojo
fn apply_capsule_damage_color(
    capsules: Query<&FlowerCapsule, Changed<FlowerCapsule>>,
    mut visual_query: Query<(&Name, &MeshMaterial3d<StandardMaterial>), Without<FlowerCapsule>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for capsule in capsules.iter() {
        let damage_ratio = 1.0 - (capsule.health as f32 / CAPSULE_MAX_HEALTH as f32);
        let color = Color::srgb(
            0.3 + 0.7 * damage_ratio,
            0.0,
            0.4 * (1.0 - damage_ratio),
        );

        for (name, mat_handle) in visual_query.iter_mut() {
            if name.as_str().to_lowercase().contains("corrupted_glass")
                && !name.as_str().to_lowercase().contains("collision_")
            {
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.base_color = color;
                    info!("Color daño aplicado: {:?} (salud: {})", color, capsule.health);
                }
            }
        }
    }
}

pub fn damage_capsule(
    commands: &mut Commands,
    entity: Entity,
    capsule: &mut FlowerCapsule,
    tracker: &mut CapsuleTracker,
    shards: &Query<(Entity, &ShardOwner)>,
    level_assets: &LevelAssets,   // ← nuevo
    level: Entity,                // ← nuevo
) {
    if capsule.health == 0 { return; }
    capsule.health -= 1;

    // Sonido de golpe a cápsula siempre
    commands.entity(level).with_child(sound_effect(
        level_assets.capsule_damage.clone(),
        (),
    ));

    if capsule.health == 0 {
        for (shard_entity, owner) in shards.iter() {
            if owner.0 == entity {
                commands.entity(shard_entity)
                    .insert(RigidBody::Dynamic)
                    .insert(LinearVelocity(Vec3::new(
                        rand::random::<f32>() * 4.0 - 2.0,
                        rand::random::<f32>() * 3.0 + 1.0,
                        rand::random::<f32>() * 4.0 - 2.0,
                    )));
            }
        }
        commands.entity(entity).despawn();
        tracker.broken += 1;
        info!("Cápsula destruida ({}/{})", tracker.broken, tracker.total);
    }
}

fn flower_capsule_tracking(
    tracker: Res<CapsuleTracker>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut level_won: Local<bool>,
) {
    if *level_won { return; }
    if tracker.all_broken() {
        *level_won = true;
        next_menu.set(Menu::Victory);
    }
}
