use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::game::components::{Health, Ship};
use crate::game::resources::{CurrentTurn, LocalPlayerId, OrderQueue, Selection};
use crate::game::state::GameState;

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct StateText;

#[derive(Component)]
pub struct RosterText;

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(14.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                TurnText,
                Text::new("Turn 0"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                StateText,
                Text::new("Connecting…"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(Color::srgb(0.75, 0.85, 1.0)),
            ));
            p.spawn((
                RosterText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.65, 0.75, 0.85)),
            ));
        });
}

// ── Update systems ────────────────────────────────────────────────────────────

pub fn update_turn_text(
    current_turn: Res<CurrentTurn>,
    mut query: Query<&mut Text, With<TurnText>>,
) {
    if !current_turn.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        text.0 = format!("Turn {}", current_turn.0);
    }
}

pub fn update_state_text(
    state: Res<State<GameState>>,
    selection: Res<Selection>,
    orders: Res<OrderQueue>,
    local_id: Res<LocalPlayerId>,
    mut query: Query<&mut Text, With<StateText>>,
) {
    if !state.is_changed()
        && !selection.is_changed()
        && !orders.is_changed()
        && !local_id.is_changed()
    {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        text.0 = match state.get() {
            GameState::Planning => {
                if local_id.0.is_none() {
                    "Connecting…".to_string()
                } else if selection.0.is_some() {
                    "Ship selected — click a hex to move  |  Esc to cancel".to_string()
                } else if orders.orders.is_empty() {
                    "Planning — click a ship to select  |  Space to submit".to_string()
                } else {
                    format!(
                        "Planning — {} order(s) queued  |  Space to submit",
                        orders.orders.len()
                    )
                }
            }
            GameState::Resolving => "Resolving…".to_string(),
            GameState::Animating => "Animating…".to_string(),
        };
    }
}

pub fn update_roster_text(
    ships: Query<(&Ship, &Health)>,
    local_id: Res<LocalPlayerId>,
    mut query: Query<&mut Text, With<RosterText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };

    // owner_id → (ship count, total hp)
    let mut roster: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
    for (ship, health) in &ships {
        let e = roster.entry(ship.owner_id).or_default();
        e.0 += 1;
        e.1 += health.0;
    }

    if roster.is_empty() {
        text.0.clear();
        return;
    }

    let lines: Vec<String> = roster
        .iter()
        .map(|(&id, &(ship_count, total_hp))| {
            let you = if local_id.0 == Some(id) { " ◀ you" } else { "" };
            format!("P{id}  {ship_count} ship(s)  HP {total_hp}{you}")
        })
        .collect();
    text.0 = lines.join("\n");
}
