use rltk::RandomNumberGenerator;

use crate::components::Position;
use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

#[allow(dead_code)]
pub enum XStart {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
pub enum YStart {
    Top,
    Center,
    Bottom,
}

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl AreaStartingPosition {
    #[allow(dead_code)]
    pub fn new(x: XStart, y: YStart) -> Box<AreaStartingPosition> {
        Box::new(AreaStartingPosition { x, y })
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let seed_x = match self.x {
            XStart::Left => 1,
            XStart::Center => build_data.map.width / 2,
            XStart::Right => build_data.map.width - 2,
        };

        let seed_y = match self.y {
            YStart::Top => 1,
            YStart::Center => build_data.map.height / 2,
            YStart::Bottom => build_data.map.height - 2,
        };

        let mut available_floors: Vec<(usize, f32)> = build_data
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile_type)| **tile_type == TileType::Floor)
            .map(|(idx, _)| {
                // TODO(DP): use idx -> (x, y) function
                let map_point = rltk::Point::new(
                    idx as i32 % build_data.map.width,
                    idx as i32 / build_data.map.width,
                );
                let seed_point = rltk::Point::new(seed_x, seed_y);
                let distance =
                    rltk::DistanceAlg::PythagorasSquared.distance2d(map_point, seed_point);

                (idx, distance)
            })
            .collect();

        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|(_, left_distance), (_, right_distance)| {
            left_distance.partial_cmp(right_distance).unwrap()
        });

        let (start_max_idx, _) = available_floors.first().unwrap();
        // TODO(DP): use idx -> (x, y) function
        let x = *start_max_idx as i32 % build_data.map.width;
        let y = *start_max_idx as i32 / build_data.map.width;

        build_data.starting_position = Some(Position { x, y });
    }
}
