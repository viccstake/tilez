use bevy::prelude::*;

use crate::game::components::{AnimTimer, Position, TargetPosition};
use crate::game::resources::HexLayout;
use crate::game::state::GameState;

const ANIM_DURATION: f32 = 0.4;

/// Lerps each ship's `Transform` from its position when animation began toward
/// its `TargetPosition` over `ANIM_DURATION` seconds.
///
/// On the first frame a ship has `TargetPosition` but no `AnimTimer`, the timer
/// is initialised with the current `Transform` as the start.  When the lerp
/// completes, `Position` is updated, both components are removed, and — once
/// every ship is done — the state transitions back to `Planning`.
pub fn animate_transitions(
    mut commands: Commands,
    time: Res<Time>,
    hex_layout: Res<HexLayout>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Position,
        &TargetPosition,
        Option<&mut AnimTimer>,
    )>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut total = 0;
    let mut completed = 0;

    for (entity, mut transform, mut pos, target, maybe_timer) in &mut query {
        total += 1;
        let end = target.hex.to_world(hex_layout.size);

        if let Some(mut timer) = maybe_timer {
            timer.elapsed += time.delta_secs();
            let t = (timer.elapsed / timer.duration).min(1.0);
            let lerped = timer.start_world.lerp(end, t);
            transform.translation.x = lerped.x;
            transform.translation.y = lerped.y;

            if t >= 1.0 {
                pos.hex = target.hex;
                commands.entity(entity).remove::<(TargetPosition, AnimTimer)>();
                completed += 1;
            }
        } else {
            // First frame: record current world position as the lerp start.
            commands.entity(entity).insert(AnimTimer {
                elapsed: 0.0,
                duration: ANIM_DURATION,
                start_world: transform.translation.truncate(),
            });
        }
    }

    if total > 0 && total == completed {
        next_state.set(GameState::Planning);
    }
}
