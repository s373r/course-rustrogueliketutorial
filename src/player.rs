use super::{xy_idx, Player, Position, State, TileType};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let new_x = (pos.x + delta_x).clamp(0, 79);
        let new_y = (pos.y + delta_y).clamp(0, 79);
        let destination_idx = xy_idx(new_x, new_y);

        if map[destination_idx] != TileType::Wall {
            pos.x = new_x;
            pos.y = new_y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    let optional_player_movement = {
        use VirtualKeyCode::*;

        ctx.key.and_then(|key| match key {
            Left | H | A => Some((-1, 0)),
            Right | L | D => Some((1, 0)),
            Up | K | W => Some((0, -1)),
            Down | J | S => Some((0, 1)),
            _ => None,
        })
    };

    if let Some((delta_x, delta_y)) = optional_player_movement {
        try_move_player(delta_x, delta_y, &mut gs.ecs);
    }
}
