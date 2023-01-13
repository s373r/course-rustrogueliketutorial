mod common;
mod simple_map;

use specs::World;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self, new_depth: i32) -> (Map, Position);
    fn spawn_entities(&mut self, map: &Map, ecs: &mut World, new_depth: i32);
}

pub fn random_builder() -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slighlty random
    Box::new(SimpleMapBuilder {})
}
