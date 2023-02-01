use rltk::console;
use specs::prelude::*;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::remove_unreachable_areas_returning_most_distant;
use crate::map_builders::MapBuilder;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&self, ecs: &mut World) {
        for (map_idx, entity_name) in self.spawns.iter() {
            spawner::spawn_entity(ecs, &(map_idx, entity_name));
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

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel {
                template: "../resources/wfc-populated.xp",
            },
            spawns: Vec::new(),
        }
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();

                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);

                        match (cell.ch as u8) as char {
                            ' ' => self.map.tiles[idx] = TileType::Floor, // space
                            '#' => self.map.tiles[idx] = TileType::Wall,  // #
                            '@' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.starting_position = Position {
                                    x: x as i32,
                                    y: y as i32,
                                };
                            }
                            '>' => self.map.tiles[idx] = TileType::DownStairs,
                            'g' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Goblin".to_string()));
                            }
                            'o' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Orc".to_string()));
                            }
                            '^' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Bear Trap".to_string()));
                            }
                            '%' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Rations".to_string()));
                            }
                            '!' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Health Potion".to_string()));
                            }
                            _ => {
                                console::log(format!(
                                    "Unknown glyph loading map: {}",
                                    (cell.ch as u8) as char
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template),
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        if self.starting_position.x == 0 {
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
            self.take_snapshot();

            // Find all tiles we can reach from the starting point
            let exit_tile =
                remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }
}
