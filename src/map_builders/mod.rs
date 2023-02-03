mod area_starting_points;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
pub mod common;
mod cull_unreachable;
mod distant_exit;
mod dla;
mod drunkard;
mod maze;
mod prefab_builder;
mod room_based_spawner;
mod room_based_stairs;
mod room_based_starting_position;
mod room_corner_rounding;
mod room_exploder;
mod simple_map;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;

use specs::World;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::area_starting_points::{AreaStartingPosition, XStart, YStart};
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::cull_unreachable::CullUnreachable;
use crate::map_builders::distant_exit::DistantExit;
use crate::map_builders::dla::DLABuilder;
use crate::map_builders::drunkard::DrunkardsWalkBuilder;
use crate::map_builders::maze::MazeBuilder;
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::map_builders::room_based_spawner::RoomBasedSpawner;
use crate::map_builders::room_based_stairs::RoomBasedStairs;
use crate::map_builders::room_based_starting_position::RoomBasedStartingPosition;
use crate::map_builders::room_corner_rounding::RoomCornerRounder;
use crate::map_builders::room_exploder::RoomExploder;
use crate::map_builders::simple_map::SimpleMapBuilder;
use crate::map_builders::voronoi::VoronoiCellBuilder;
use crate::map_builders::voronoi_spawning::VoronoiSpawning;
use crate::map_builders::waveform_collapse::WaveformCollapseBuilder;
use crate::rect::Rect;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
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

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
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

fn random_initial_builder(
    rng: &mut rltk::RandomNumberGenerator,
) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 17);

    match builder {
        1 => (BspDungeonBuilder::new(), true),
        2 => (BspInteriorBuilder::new(), true),
        3 => (CellularAutomataBuilder::new(), false),
        4 => (DrunkardsWalkBuilder::open_area(), false),
        5 => (DrunkardsWalkBuilder::open_halls(), false),
        6 => (DrunkardsWalkBuilder::winding_passages(), false),
        7 => (DrunkardsWalkBuilder::fat_passages(), false),
        8 => (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => (MazeBuilder::new(), false),
        10 => (DLABuilder::walk_inwards(), false),
        11 => (DLABuilder::walk_outwards(), false),
        12 => (DLABuilder::central_attractor(), false),
        13 => (DLABuilder::insectoid(), false),
        14 => (VoronoiCellBuilder::pythagoras(), false),
        15 => (VoronoiCellBuilder::manhattan(), false),
        16 => (
            PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED),
            false,
        ),
        _ => (SimpleMapBuilder::new(), true),
    }
}

pub fn random_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);

    builder.start_with(BspDungeonBuilder::new());
    builder.with(RoomCornerRounder::new());
    builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.with(CullUnreachable::new());
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());

    builder

    // TODO(DP): return after tests
    // let mut builder = BuilderChain::new(new_depth);
    //
    // let (random_starter, has_rooms) = random_initial_builder(rng);
    //
    // builder.start_with(random_starter);
    //
    // if has_rooms {
    //     builder.with(RoomBasedSpawner::new());
    //     builder.with(RoomBasedStairs::new());
    //     builder.with(RoomBasedStartingPosition::new());
    // } else {
    //     builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    //     builder.with(CullUnreachable::new());
    //     builder.with(VoronoiSpawning::new());
    //     builder.with(DistantExit::new());
    // }
    //
    // if rng.roll_dice(1, 3) == 1 {
    //     builder.with(WaveformCollapseBuilder::new());
    // }
    //
    // if rng.roll_dice(1, 20) == 1 {
    //     builder.with(PrefabBuilder::sectional(
    //         prefab_builder::prefab_sections::UNDERGROUND_FORT,
    //     ));
    // }
    //
    // builder.with(PrefabBuilder::vaults());
    //
    // builder
}
