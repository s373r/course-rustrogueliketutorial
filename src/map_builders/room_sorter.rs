use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct RoomSorter {}

impl MetaMapBuilder for RoomSorter {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.sorter(rng, build_data);
    }
}

impl RoomSorter {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomSorter> {
        Box::new(RoomSorter {})
    }

    fn sorter(&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data
            .rooms
            .as_mut()
            .unwrap()
            .sort_by(|a, b| a.x1.cmp(&b.x1));
    }
}
