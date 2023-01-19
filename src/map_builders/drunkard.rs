use rltk::{console, RandomNumberGenerator};
use specs::prelude::*;
use std::collections::HashMap;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
};
use crate::map_builders::MapBuilder;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
}

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if !SHOW_MAPGEN_VISUALIZER {
            return;
        }

        let mut snapshot = self.map.clone();

        snapshot.revealed_tiles.fill(true);

        self.history.push(snapshot);
    }
}

impl DrunkardsWalkBuilder {
    pub fn new(new_depth: i32, settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Set a central starting point
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);

        self.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (total_tiles / 2) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut digger_count = 0;
        let mut active_digger_count = 0;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x = self.starting_position.x;
            let mut drunk_y = self.starting_position.y;
            let mut drunk_life = 400;

            while drunk_life > 0 {
                let drunk_idx = self.map.xy_idx(drunk_x, drunk_y);

                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }

                self.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);

                match stagger_direction {
                    1 => {
                        if drunk_x > 2 {
                            drunk_x -= 1;
                        }
                    }
                    2 => {
                        if drunk_x < self.map.width - 2 {
                            drunk_x += 1;
                        }
                    }
                    3 => {
                        if drunk_y > 2 {
                            drunk_y -= 1;
                        }
                    }
                    _ => {
                        if drunk_y < self.map.height - 2 {
                            drunk_y += 1;
                        }
                    }
                }

                drunk_life -= 1;
            }

            if did_something {
                self.take_snapshot();

                active_digger_count += 1;
            }

            digger_count += 1;

            for tile in self.map.tiles.iter_mut() {
                if *tile == TileType::DownStairs {
                    *tile = TileType::Floor;
                }
            }

            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }

        console::log(format!(
            "{digger_count} dwarves gave up their sobriety, of whom {active_digger_count} actually found a wall."
        ));
    }
}
