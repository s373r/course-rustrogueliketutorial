use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomDrawer {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomDrawer> {
        Box::new(RoomDrawer {})
    }

    fn build(&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let Some(rooms_builder) = &build_data.rooms else {
            panic!("Room Rounding require a builder with room structures");
        };
        let rooms = rooms_builder.clone();

        for room in rooms.iter() {
            for y in room.y1 + 1..=room.y2 {
                for x in room.x1 + 1..=room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if build_data.map.is_valid_idx(idx) {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }
    }
}
