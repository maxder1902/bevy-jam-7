use bevy::prelude::*;

pub struct HideCollidersPlugin;

impl Plugin for HideCollidersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hide_collision_geometry);
    }
}

fn hide_collision_geometry(
    mut commands: Commands,
    // Buscamos entidades que tengan nombre y visibilidad, y que no hayamos procesado antes
    query: Query<(Entity, &Name, &Visibility), (Added<Name>, Without<Node>)>,
) {
    for (entity, name, visibility) in query.iter() {
        let name_str = name.as_str();

        // Comprobamos si el nombre empieza por los prefijos que definiste
        if name_str.starts_with("Collider") || name_str.starts_with("Collision") {

            // Opción 1: Simplemente ocultarlo (sigue existiendo para la física/navmesh)
            commands.entity(entity).insert(Visibility::Hidden);

            // Opción 2 (Opcional): Si quieres estar 100% seguro de que no consume recursos de GPU,
            // puedes quitarle el componente de malla, pero manteniendo la entidad para Avian:
            // commands.entity(entity).remove::<Mesh3d>();

            info!("Ocultando objeto de colisión: {}", name_str);
        }
    }
}
