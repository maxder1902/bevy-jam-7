//! Plays animations from a skinned glTF.

use std::{f32::consts::PI, time::Duration};

use bevy::{animation::RepeatAnimation, light::CascadeShadowConfigBuilder, prelude::*};

use crate::screens::gameplay::player::Player;

const KATANA_PATH: &str = "models/katana.glb";

#[derive(Resource)]
pub struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

pub fn katana_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    player: Single<Entity, With<Camera3d>>,
) {
    // Build the animation graph
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(KATANA_PATH)), // idle
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(KATANA_PATH)), // r_swing
    ]);

    // Keep our animation graph in a Resource so that it can be inserted onto
    // the correct entity once the scene actually loads.
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });

    commands.spawn((
        Name::new("Katana"),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(KATANA_PATH))),
        Transform::from_translation(Vec3::new(-0.1, -0.8, -1.4))
            .with_rotation(Quat::from_rotation_y(0.05))
            .with_scale(Vec3::splat(0.8)),
        ChildOf(player.entity()),
    ));
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
pub fn poor_setup_for_katana_animations(
    // TODO: system from bevy example, idk how to make it non update
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}

pub fn katana_animation(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut non_idle: Local<bool>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        if mouse_input.just_pressed(MouseButton::Left) {
            transitions
                .play(
                    &mut player,
                    animations.animations[1],
                    Duration::from_millis(60),
                )
                .set_speed(1.3);
            *non_idle = true;
        }

        if player.all_finished() {
            *non_idle = false;
            transitions
                .play(
                    &mut player,
                    animations.animations[0],
                    Duration::from_millis(250),
                )
                .repeat();
        }
    }
}
