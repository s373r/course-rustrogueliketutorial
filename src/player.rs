use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::map::{Map, TileType};
use crate::{RunState, State};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();

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

        let new_x = (pos.x + delta_x).clamp(0, map.width);
        let new_y = (pos.y + delta_y).clamp(0, map.height);
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

            if let Some(door) = doors.get_mut(*potential_target) {
                door.open = true;

                blocks_visibility.remove(*potential_target);
                blocks_movement.remove(*potential_target);

                let glyph = renderables.get_mut(*potential_target).unwrap();

                glyph.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = new_x;
            pos.y = new_y;

            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");

            let mut player_pos = ecs.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;

    let optional_player_movement = match ctx.key {
        Some(key) => match key {
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
            Space => return skip_turn(&mut gs.ecs),
            //
            G => {
                get_item(&mut gs.ecs);
                return RunState::PlayerTurn;
            }
            I => return RunState::ShowInventory,
            F => return RunState::ShowDropItem,
            R => return RunState::ShowRemoveItem,
            //
            Escape => return RunState::SaveGame,
            // Level changes
            Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }

                None
            }
            //
            _ => None,
        },
        None => None,
    };

    if let Some((delta_x, delta_y)) = optional_player_movement {
        try_move_player(delta_x, delta_y, &mut gs.ecs);

        RunState::PlayerTurn
    } else {
        RunState::AwaitingInput
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut game_log = ecs.fetch_mut::<GameLog>();

    let mut target_item = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => game_log
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();

            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);

    if map.tiles[player_idx] != TileType::DownStairs {
        let mut game_log = ecs.fetch_mut::<GameLog>();

        game_log
            .entries
            .push("There is no way down from here.".to_string());
        return false;
    }

    true
}

fn skip_turn(ecs: &mut World) -> RunState {
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let player_entity = ecs.fetch::<Entity>();
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    let map_resource = ecs.fetch::<Map>();
    let monsters = ecs.read_storage::<Monster>();
    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let mut can_heal = true;

    if let Some(HungerClock { state, .. }) = hunger_clocks.get(*player_entity) {
        match state {
            HungerState::Hungry | HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    if !can_heal {
        'outer: for tile in viewshed.visible_tiles.iter() {
            let idx = map_resource.xy_idx(tile.x, tile.y);

            for entity_id in map_resource.tile_content[idx].iter() {
                let has_mob = monsters.get(*entity_id).is_some();

                if has_mob {
                    can_heal = false;
                    break 'outer;
                }
            }
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();

        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}
