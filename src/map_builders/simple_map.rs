use crate::map::Map;
use crate::map_builders::MapBuilder;

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> Map {
        Map::new(new_depth)
    }
}
