use bevy::prelude::*;
use crate::screens::Screen;
use crate::screens::gameplay::player::Player;

pub struct FallDeathPlugin;

/// Si el jugador cae por debajo de esta altura, muere instantáneamente
const DEATH_Y: f32 = -200.0;

impl Plugin for FallDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            fall_death.run_if(in_state(Screen::Gameplay)),
        );
    }
}

fn fall_death(
    mut player: Single<(&Transform, &mut Player)>,
) {
    let (transform, mut player) = player.into_inner();
    if transform.translation.y < DEATH_Y {
        player.health = 0.0;
        info!("Player cayó al vacío — muerte instantánea");
    }
}
