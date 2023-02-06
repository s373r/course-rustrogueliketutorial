use crate::map::Map;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub enum RoomSort {
    Leftmost,
    Rightmost,
    Topmost,
    Bottommost,
    Central,
}

pub struct RoomSorter {
    sort_by: RoomSort,
}

impl MetaMapBuilder for RoomSorter {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.sorter(rng, build_data);
    }
}

impl RoomSorter {
    #[allow(dead_code)]
    pub fn new(sort_by: RoomSort) -> Box<RoomSorter> {
        Box::new(RoomSorter { sort_by })
    }

    fn sorter(&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = build_data.rooms.as_mut().unwrap();

        match self.sort_by {
            RoomSort::Leftmost => rooms.sort_by(|a, b| a.x1.cmp(&b.x1)),
            RoomSort::Rightmost => rooms.sort_by(|a, b| b.x2.cmp(&a.x2)),
            RoomSort::Topmost => rooms.sort_by(|a, b| a.y1.cmp(&b.y1)),
            RoomSort::Bottommost => rooms.sort_by(|a, b| b.y2.cmp(&a.y2)),
            RoomSort::Central => {
                let Map { width, height, .. } = build_data.map;
                let map_center = rltk::Point::new(width / 2, height / 2);

                rooms.sort_by(|a, b| {
                    let a_center = {
                        let (x, y) = a.center();
                        rltk::Point::new(x, y)
                    };
                    let b_center = {
                        let (x, y) = b.center();
                        rltk::Point::new(x, y)
                    };
                    let distance_a = rltk::DistanceAlg::Pythagoras.distance2d(a_center, map_center);
                    let distance_b = rltk::DistanceAlg::Pythagoras.distance2d(b_center, map_center);

                    distance_a.partial_cmp(&distance_b).unwrap()
                });
            }
        }
    }
}
