use bevy::prelude::*;
use crate::screens::Screen;
use super::hud::KillStreak;

pub struct ComboSystemPlugin;

impl Plugin for ComboSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SlowMotionState::default());
        app.add_systems(
            Update,
            (
                streak_tracking,
                tick_slow_motion,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

#[derive(Resource, Default)]
pub struct SlowMotionState {
    pub active: bool,
    pub timer: f32,
    last_milestone: u32,
}

// Detecta múltiplos de 3 kills y activa slow motion
fn streak_tracking(
    streak: Res<KillStreak>,
    mut slow_mo: ResMut<SlowMotionState>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    // Si el streak se resetea, resetear el milestone
    if streak.kills == 0 {
        slow_mo.last_milestone = 0;
        return;
    }

    let current_milestone = (streak.kills / 3) * 3;

    if current_milestone > slow_mo.last_milestone && streak.kills >= 3 {
        // Activar slow motion
        slow_mo.active = true;
        slow_mo.timer = 0.4;
        slow_mo.last_milestone = current_milestone;

        // Ralentizar el tiempo a 30% de velocidad
        virtual_time.set_relative_speed(0.3);
    }
}

// Cuenta el tiempo de slow motion y restaura velocidad normal
fn tick_slow_motion(
    mut slow_mo: ResMut<SlowMotionState>,
    mut virtual_time: ResMut<Time<Virtual>>,
    time: Res<Time<Real>>,
) {
    if !slow_mo.active {
        return;
    }

    slow_mo.timer -= time.delta_secs();

    if slow_mo.timer <= 0.0 {
        slow_mo.active = false;
        // Restaurar velocidad normal
        virtual_time.set_relative_speed(1.0);
    }
}
