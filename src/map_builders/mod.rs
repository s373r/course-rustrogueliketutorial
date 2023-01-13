mod common;
mod simple_map;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(new_depth: i32) -> (Map, Position);
}

pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(new_depth)
}
