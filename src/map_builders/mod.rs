mod bsp_dungeon;
mod common;
mod simple_map;

use specs::World;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slighlty random
    Box::new(BspDungeonBuilder::new(new_depth))
}
