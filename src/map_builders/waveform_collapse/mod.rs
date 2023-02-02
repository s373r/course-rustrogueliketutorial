mod common;
mod constraints;
mod solver;

use rltk::RandomNumberGenerator;
use std::collections::HashMap;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::*;
use crate::map_builders::waveform_collapse::common::{patterns_to_constraints, MapChunk};
use crate::map_builders::waveform_collapse::constraints::{build_patterns, render_pattern_to_map};
use crate::map_builders::waveform_collapse::solver::Solver;
use crate::map_builders::MapBuilder;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER, SHOW_MAP_AFTER_GENERATION};

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    derive_from: Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<SpawnEntity>,
}

impl MapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self) {
        self.build();
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

    fn get_spawn_list(&self) -> &Vec<SpawnEntity> {
        &self.spawn_list
    }
}

impl WaveformCollapseBuilder {
    fn new(new_depth: i32, derive_from: Option<Box<dyn MapBuilder>>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            derive_from,
            spawn_list: Vec::new(),
        }
    }

    pub fn derived_map(new_depth: i32, builder: Box<dyn MapBuilder>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder::new(new_depth, Some(builder))
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        const CHUNK_SIZE: i32 = 8;

        let pre_builder = &mut self.derive_from.as_mut().unwrap();

        pre_builder.build_map();

        self.map = pre_builder.get_map();

        for t in self.map.tiles.iter_mut() {
            if *t == TileType::DownStairs {
                *t = TileType::Floor;
            }
        }

        self.take_snapshot();

        let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);

        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        self.map = Map::new(self.depth);

        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.take_snapshot();
            }

            self.take_snapshot();

            if solver.possible {
                break;
            }

            // If it has hit an impossible condition, try again
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);

        /*while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }*/

        self.take_snapshot();

        let exit_tile = if !SHOW_MAP_AFTER_GENERATION {
            // Find all tiles we can reach from the starting point
            remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx)
        } else {
            0
        };

        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;

        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // Spawn the entities
        for area in self.noise_areas.iter() {
            spawner::spawn_region(
                &self.map,
                &mut rng,
                area.1,
                self.depth,
                &mut self.spawn_list,
            );
        }
    }

    fn render_tile_gallery(&mut self, patterns: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(0);

        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;

        while counter < patterns.len() {
            render_pattern_to_map(&mut self.map, &patterns[counter], chunk_size, x, y);

            x += chunk_size + 1;

            if x + chunk_size > self.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > self.map.height {
                    // Move to the next page
                    self.take_snapshot();
                    self.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }

            counter += 1;
        }

        self.take_snapshot();
    }
}
