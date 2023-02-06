use crate::map_builders::common::{apply_horizontal_tunnel, apply_vertical_tunnel, Corridor};
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct DoglegCorridors {}

impl MetaMapBuilder for DoglegCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl DoglegCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<DoglegCorridors> {
        Box::new(DoglegCorridors {})
    }

    fn corridors(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let Some(rooms_builder) = &build_data.rooms else {
            panic!("Room Explosions require a builder with room structures");
        };
        let rooms = rooms_builder.clone();

        let mut corridors: Vec<Corridor> = Vec::new();
        for (i, room) in rooms.iter().enumerate().skip(1) {
            let (new_x, new_y) = room.center();
            let (prev_x, prev_y) = rooms[i - 1].center();

            let (mut corridor1, mut corridor2) = if rng.range(0, 2) == 1 {
                (
                    apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y),
                    apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x),
                )
            } else {
                (
                    apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x),
                    apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y),
                )
            };
            let corridor = {
                corridor1.append(&mut corridor2);
                corridor1
            };

            corridors.push(corridor);
            build_data.take_snapshot();
        }
        build_data.corridors = Some(corridors);
    }
}
