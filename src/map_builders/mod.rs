mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod drunkard;
mod maze;
mod simple_map;

use rltk::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::drunkard::DrunkardsWalkBuilder;
use crate::map_builders::maze::MazeBuilder;
use crate::map_builders::simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // TODO(DP): uncomment after tests
    // let mut rng = RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 7);
    //
    // match builder {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // }

    Box::new(MazeBuilder::new(new_depth))
}
