use rltk::{field_of_view, Point};
use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, BlocksVisibility>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            entities,
            mut viewshed,
            pos,
            player,
            mut hidden,
            mut rng,
            mut game_log,
            names,
            blocks_visibility,
        ) = data;

        map.view_blocked.clear();
        for (block_pos, _) in (&pos, &blocks_visibility).join() {
            let idx = map.xy_idx(block_pos.x, block_pos.y);

            map.view_blocked.insert(idx);
        }

        // TODO(DP): use early ~~returns~~ continues
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if !viewshed.dirty {
                continue;
            }

            viewshed.dirty = false;
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            // If this is the player, reveal what they can see
            if player.get(ent).is_none() {
                continue;
            }

            map.visible_tiles.fill(false);

            for vis in viewshed.visible_tiles.iter() {
                let idx = map.xy_idx(vis.x, vis.y);
                map.revealed_tiles[idx] = true;
                map.visible_tiles[idx] = true;

                // Chance to reveal hidden things
                for tile_entity in map.tile_content[idx].iter() {
                    let maybe_hidden = hidden.get(*tile_entity);

                    if maybe_hidden.is_none() {
                        continue;
                    }

                    if rng.roll_dice(1, 24) != 1 {
                        continue;
                    }

                    if let Some(name) = names.get(*tile_entity) {
                        game_log
                            .entries
                            .push(format!("You spotted a {}.", &name.name));
                    }

                    hidden.remove(*tile_entity);
                }
            }
        }
    }
}
