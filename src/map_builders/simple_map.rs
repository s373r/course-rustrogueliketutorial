use rltk::RandomNumberGenerator;

use crate::map_builders::common::*;
use crate::map_builders::{BuilderMap, InitialMapBuilder};
use crate::rect::Rect;

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.rooms_and_corridors(rng, build_data);
    }
}

impl SimpleMapBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder {})
    }

    pub fn rooms_and_corridors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rooms: Vec<Rect> = Vec::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let has_intersect_other_rooms = rooms
                .iter()
                .any(|another_room| new_room.intersect(another_room));

            if has_intersect_other_rooms {
                continue;
            }

            apply_room(&mut build_data.map, &new_room);

            build_data.take_snapshot();

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms.last().unwrap().center();

                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y);
                }
            }

            rooms.push(new_room);

            build_data.take_snapshot();
        }

        build_data.rooms = Some(rooms);
    }
}
