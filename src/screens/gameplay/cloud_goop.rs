use bevy::prelude::*;
use crate::screens::Screen;
use crate::visuals::goop::{GoopMaterial, GoopMaterialExtention};
use bevy::pbr::ExtendedMaterial;

pub struct CloudGoopPlugin;

impl Plugin for CloudGoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_goop_to_clouds,
                animate_cloud_goop,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

/// Marca los meshes que ya tienen GoopMaterial para animarlos
#[derive(Component)]
pub struct CloudGoopAnimated;

/// Detecta meshes con nombre "big_bad_cloud" cuando cargan y reemplaza su material por GoopMaterial
fn apply_goop_to_clouds(
    mut commands: Commands,
    new_meshes: Query<
        (Entity, &Name, &MeshMaterial3d<StandardMaterial>),
        Added<MeshMaterial3d<StandardMaterial>>,
    >,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut goop_materials: ResMut<Assets<GoopMaterial>>,
) {
    for (entity, name, std_mat_handle) in new_meshes.iter() {
        if !name.as_str().to_lowercase().contains("big_bad_cloud") {
            continue;
        }

        let base = std_materials
            .get(&std_mat_handle.0)
            .cloned()
            .unwrap_or_default();

        let goop_handle = goop_materials.add(ExtendedMaterial {
            base,
            extension: GoopMaterialExtention::new(2.0, 0.0),
        });

        commands.entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((MeshMaterial3d(goop_handle), CloudGoopAnimated));

        info!("GoopMaterial aplicado a: {}", name);
    }
}

/// Anima el extent del goop suavemente en loop para dar sensación de nube viva
fn animate_cloud_goop(
    time: Res<Time>,
    animated: Query<&MeshMaterial3d<GoopMaterial>, With<CloudGoopAnimated>>,
    mut goop_materials: ResMut<Assets<GoopMaterial>>,
) {
    for mat_handle in animated.iter() {
        if let Some(mat) = goop_materials.get_mut(&mat_handle.0) {
            let t = time.elapsed_secs();
            mat.extension.extent = 3.0 + 3.0 * (t * 0.5).sin();
        }
    }
}
