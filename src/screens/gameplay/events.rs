use bevy::prelude::*;

struct EnemyKilledEvent;
struct CapsuleDestroyedEvent;
struct ArenaCollapsedEvent;
struct ComboTriggeredEvent;
struct LevelWonEvent;
struct LevelLostEvent;

#[derive(Event)]
pub struct SpawnAlarmClockEvent {
    pub position: Vec3,
}
