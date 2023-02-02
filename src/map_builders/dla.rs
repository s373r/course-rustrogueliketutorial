use rltk::RandomNumberGenerator;
use std::collections::HashMap;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::*;
use crate::map_builders::MapBuilder;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

const MAP_GENERATION_SAVE_AFTER_SNAPSHOTS_COUNT: i32 = 25;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
    spawn_list: Vec<SpawnEntity>,
}

impl DLABuilder {
    pub fn walk_inwards(new_depth: i32) -> DLABuilder {
        Self::new(new_depth, DLAAlgorithm::WalkInwards, 1, Symmetry::None)
    }

    pub fn walk_outwards(new_depth: i32) -> DLABuilder {
        Self::new(new_depth, DLAAlgorithm::WalkOutwards, 2, Symmetry::None)
    }

    pub fn central_attractor(new_depth: i32) -> DLABuilder {
        Self::new(new_depth, DLAAlgorithm::CentralAttractor, 2, Symmetry::None)
    }

    pub fn insectoid(new_depth: i32) -> DLABuilder {
        Self::new(
            new_depth,
            DLAAlgorithm::CentralAttractor,
            2,
            Symmetry::Horizontal,
        )
    }

    fn new(
        new_depth: i32,
        algorithm: DLAAlgorithm,
        brush_size: i32,
        symmetry: Symmetry,
    ) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm,
            brush_size,
            symmetry,
            floor_percent: 0.25,
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Carve a starting seed
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);

        self.take_snapshot();

        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        // TODO(DP): extract to a function -- y - 1 and y + 1
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut i = 0;

        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y);

                    while self.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;

                        let stagger_direction = rng.roll_dice(1, 4);

                        match stagger_direction {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }

                        digger_idx = self.map.xy_idx(digger_x, digger_y);
                    }

                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
                }
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = self.starting_position.x;
                    let mut digger_y = self.starting_position.y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y);

                    while self.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);

                        match stagger_direction {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }

                        digger_idx = self.map.xy_idx(digger_x, digger_y);
                    }

                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        digger_x,
                        digger_y,
                    );
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y);

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham,
                        rltk::Point::new(digger_x, digger_y),
                        rltk::Point::new(self.starting_position.x, self.starting_position.y),
                    );

                    while self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = self.map.xy_idx(digger_x, digger_y);
                    }

                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
                }
            }

            if i % MAP_GENERATION_SAVE_AFTER_SNAPSHOTS_COUNT == 0 {
                self.take_snapshot();
            }

            i += 1;

            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }

        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);

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
}

impl MapBuilder for DLABuilder {
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