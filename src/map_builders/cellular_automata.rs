use rltk::RandomNumberGenerator;

use crate::map::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder};

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomataBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder {})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);

                if roll > 55 {
                    build_data.map.tiles[idx] = TileType::Floor
                } else {
                    build_data.map.tiles[idx] = TileType::Wall
                }
            }
        }
        build_data.take_snapshot();

        // Now we iteratively apply cellular automata rules
        for _ in 0..15 {
            let mut new_tiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height - 1 {
                for x in 1..build_data.map.width - 1 {
                    let idx = build_data.map.xy_idx(x, y);
                    let mut neighbors = 0;

                    let tiles = &build_data.map.tiles;
                    let map_width = build_data.map.width as usize;

                    if tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx - map_width] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx + map_width] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx - (map_width - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx - (map_width + 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx + (map_width - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if tiles[idx + (map_width + 1)] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        new_tiles[idx] = TileType::Wall;
                    } else {
                        new_tiles[idx] = TileType::Floor;
                    }
                }
            }

            build_data.map.tiles = new_tiles.clone();
            build_data.take_snapshot();
        }
    }
}
