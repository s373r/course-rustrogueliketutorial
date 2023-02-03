mod area_starting_points;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
pub mod common;
mod cull_unreachable;
mod dla;
mod drunkard;
mod maze;
mod prefab_builder;
mod room_based_spawner;
mod room_based_stairs;
mod room_based_starting_position;
mod simple_map;
mod voronoi;
mod waveform_collapse;

use rltk::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::common::SpawnEntity;
use crate::map_builders::dla::DLABuilder;
use crate::map_builders::drunkard::DrunkardsWalkBuilder;
use crate::map_builders::maze::MazeBuilder;
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::map_builders::room_based_spawner::RoomBasedSpawner;
use crate::map_builders::room_based_stairs::RoomBasedStairs;
use crate::map_builders::room_based_starting_position::RoomBasedStartingPosition;
use crate::map_builders::simple_map::SimpleMapBuilder;
use crate::map_builders::voronoi::VoronoiCellBuilder;
use crate::map_builders::waveform_collapse::WaveformCollapseBuilder;
use crate::rect::Rect;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if !SHOW_MAPGEN_VISUALIZER {
            return;
        }

        let mut snapshot = self.map.clone();

        snapshot.revealed_tiles.fill(true);

        self.history.push(snapshot);
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

// TODO(DP): use fluent interface for methods
impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        if self.starter.is_some() {
            panic!("You can only have one starting builder.")
        }

        self.starter = Some(starter);
    }

    pub fn with(&mut self, meta_builder: Box<dyn MetaMapBuilder>) {
        self.builders.push(meta_builder);
    }

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for meta_builder in self.builders.iter_mut() {
            meta_builder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for (map_idx, entity_name) in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(map_idx, entity_name));
        }
    }
}

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<SpawnEntity>;

    fn spawn_entities(&self, ecs: &mut World) {
        for (map_idx, entity_name) in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(map_idx, entity_name));
        }
    }
}

pub fn random_builder(new_depth: i32, _rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(BspInteriorBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    builder

    // TODO(DP): remove completely after migration
    // let mut rng = RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 17);
    //
    // let mut result: Box<dyn MapBuilder> = match builder {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
    //     7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
    //     8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
    //     9 => Box::new(MazeBuilder::new(new_depth)),
    //     10 => Box::new(DLABuilder::walk_inwards(new_depth)),
    //     11 => Box::new(DLABuilder::walk_outwards(new_depth)),
    //     12 => Box::new(DLABuilder::central_attractor(new_depth)),
    //     13 => Box::new(DLABuilder::insectoid(new_depth)),
    //     14 => Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
    //     15 => Box::new(VoronoiCellBuilder::manhattan(new_depth)),
    //     16 => Box::new(PrefabBuilder::constant(
    //         new_depth,
    //         prefab_builder::prefab_levels::WFC_POPULATED,
    //     )),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // };
    //
    // if rng.roll_dice(1, 3) == 1 {
    //     result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    // }
    //
    // if rng.roll_dice(1, 20) == 1 {
    //     result = Box::new(PrefabBuilder::sectional(
    //         new_depth,
    //         prefab_builder::prefab_sections::UNDERGROUND_FORT,
    //         result,
    //     ));
    // }
    //
    // result = Box::new(PrefabBuilder::vaults(new_depth, result));
    //
    // result
}
