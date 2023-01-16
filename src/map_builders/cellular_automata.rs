use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::MapBuilder;
use crate::SHOW_MAPGEN_VISUALIZER;

const MIN_ROOM_SIZE: i32 = 8;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&self, ecs: &mut World) {
        // We need to rewrite this, too.
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

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);

                if roll > 55 {
                    self.map.tiles[idx] = TileType::Floor
                } else {
                    self.map.tiles[idx] = TileType::Wall
                }
            }
        }

        self.take_snapshot();

        // Now we iteratively apply cellular automata rules
        for _ in 0..15 {
            let mut new_tiles = self.map.tiles.clone();

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbors = 0;

                    if self.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        new_tiles[idx] = TileType::Wall;
                    } else {
                        new_tiles[idx] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = new_tiles;

            self.take_snapshot();
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let mut start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);

        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;

            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
        }
    }
}
