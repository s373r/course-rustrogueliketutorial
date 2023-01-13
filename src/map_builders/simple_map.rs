use rltk::RandomNumberGenerator;

use crate::map::{Map, TileType};
use crate::map_builders::common::{apply_horizontal_tunnel, apply_room, apply_vertical_tunnel};
use crate::map_builders::MapBuilder;
use crate::rect::Rect;

pub struct SimpleMapBuilder {}

impl SimpleMapBuilder {
    pub fn rooms_and_corridors(map: &mut Map) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let has_intersect_other_rooms = map
                .rooms
                .iter()
                .any(|another_room| new_room.intersect(another_room));

            if has_intersect_other_rooms {
                continue;
            }

            apply_room(map, &new_room);

            if !map.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = map.rooms.last().unwrap().center();

                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                }
            }

            map.rooms.push(new_room);
        }

        let stairs_position = map.rooms.last().unwrap().center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);

        map.tiles[stairs_idx] = TileType::DownStairs;
    }
}

impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> Map {
        let mut map = Map::new(new_depth);

        SimpleMapBuilder::rooms_and_corridors(&mut map);

        map
    }
}
