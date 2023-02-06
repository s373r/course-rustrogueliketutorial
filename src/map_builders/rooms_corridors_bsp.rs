use crate::map_builders::common::draw_corridor;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct BspCorridors {}

impl MetaMapBuilder for BspCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl BspCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<BspCorridors> {
        Box::new(BspCorridors {})
    }

    fn corridors(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let Some(rooms_builder) = &build_data.rooms else {
            panic!("BSP Corridors require a builder with room structures");
        };
        let rooms = rooms_builder.clone();

        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next_room = rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, room.weight()) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, room.height()) - 1);
            let end_x = next_room.x1 + (rng.roll_dice(1, next_room.weight()) - 1);
            let end_y = next_room.y1 + (rng.roll_dice(1, room.height()) - 1);

            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }
    }
}
