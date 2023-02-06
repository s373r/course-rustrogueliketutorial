use crate::map::TileType;
use std::collections::HashSet;

use crate::map_builders::common::draw_corridor;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct StraightLineCorridors {}

impl MetaMapBuilder for StraightLineCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

// TODO(DP): looks same as NearestCorridors
impl StraightLineCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<StraightLineCorridors> {
        Box::new(StraightLineCorridors {})
    }

    fn corridors(&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let Some(rooms_builder) = &build_data.rooms else {
            panic!("Nearest Corridors require a builder with room structures");
        };
        let rooms = rooms_builder.clone();

        let mut connected: HashSet<usize> = HashSet::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance = Vec::new();
            let room_center = {
                let (x, y) = room.center();
                rltk::Point::new(x, y)
            };
            for (j, other_room) in rooms.iter().enumerate() {
                if i == j || connected.contains(&j) {
                    continue;
                }

                let other_center = {
                    let (x, y) = other_room.center();
                    rltk::Point::new(x, y)
                };
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(room_center, other_center);

                room_distance.push((j, distance));
            }

            if room_distance.is_empty() {
                continue;
            }

            room_distance.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

            let (closest_room_idx, _) = room_distance[0];
            let (dest_center_x, dest_center_y) = rooms[closest_room_idx].center();
            let line = rltk::line2d(
                rltk::LineAlg::Bresenham,
                room_center,
                rltk::Point::new(dest_center_x, dest_center_y),
            );
            for cell in line.iter() {
                let idx = build_data.map.xy_idx(cell.x, cell.y);
                build_data.map.tiles[idx] = TileType::Floor;
            }

            connected.insert(i);
            build_data.take_snapshot();
        }
    }
}
