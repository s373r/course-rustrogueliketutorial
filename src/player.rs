use super::{Map, Player, Position, State};
use crate::components::{CombatStats, Viewshed, WantsToMelee};
use crate::RunState;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        // NOTE(DP): I do not think we need this check
        // if pos.x + delta_x < 1
        //     || pos.x + delta_x > map.width - 1
        //     || pos.y + delta_y < 1
        //     || pos.y + delta_y > map.height - 1
        // {
        //     return;
        // }

        let new_x = (pos.x + delta_x).clamp(0, 79);
        let new_y = (pos.y + delta_y).clamp(0, 49);
        let destination_idx = map.xy_idx(new_x, new_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);

            if target.is_some() {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = new_x;
            pos.y = new_y;

            let mut player_pos = ecs.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    let optional_player_movement = {
        use VirtualKeyCode::*;

        ctx.key.and_then(|key| match key {
            Left | H | A => Some((-1, 0)),
            Right | L | D => Some((1, 0)),
            Up | K | W => Some((0, -1)),
            Down | J | S => Some((0, 1)),
            // Diagonals
            E | Y => Some((1, -1)),
            Q | U => Some((-1, -1)),
            X | N => Some((1, 1)),
            Z | B => Some((-1, 1)),
            //
            _ => None,
        })
    };

    if let Some((delta_x, delta_y)) = optional_player_movement {
        try_move_player(delta_x, delta_y, &mut gs.ecs);

        RunState::Running
    } else {
        RunState::Paused
    }
}
