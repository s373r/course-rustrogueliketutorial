use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::rect::Rect;

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
            self.circle(build_data, room);
            build_data.take_snapshot();
        }
    }

    fn rectangle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.is_valid_idx(idx) {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn circle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        let radius = i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32 / 2.0;
        let room_center = {
            let (x, y) = room.center();
            rltk::Point::new(x, y)
        };
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let room_point = rltk::Point::new(x, y);
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(room_center, room_point);
                let idx = build_data.map.xy_idx(x, y);

                if build_data.map.is_valid_idx(idx) && distance <= radius {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }
}
