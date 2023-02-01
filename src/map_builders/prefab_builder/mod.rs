mod prefab_levels;
mod prefab_sections;

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
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: prefab_levels::PrefabLevel,
    },
    Sectional {
        section: prefab_sections::PrefabSection,
    },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    // TODO(DP): do we really need this field?
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
    previous_builder: Option<Box<dyn MapBuilder>>,
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
    pub fn new(new_depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Constant {
                level: prefab_levels::WFC_POPULATED,
            },
            spawns: Vec::new(),
            previous_builder,
        }
    }

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position { x, y };
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
                console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();

                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);

                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }

    fn read_ascii_to_vec(template: &str) -> Vec<char> {
        // Start by converting to a vector, with newlines removed
        let mut string_vec: Vec<char> = template
            .chars()
            .filter(|a| *a != '\r' && *a != '\n')
            .collect();

        // NOTE(DP): Probable, we do not need this since I updated all NBSP( ) to spaces
        for c in string_vec.iter_mut() {
            if *c as u8 == 160u8 {
                *c = ' ';
            }
        }

        string_vec
    }

    #[allow(dead_code)]
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section),
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

    fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection) {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();

        prev_builder.build_map();

        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map();
        self.take_snapshot();

        use prefab_sections::*;

        let string_vec = Self::read_ascii_to_vec(section.template);

        // Place the new section
        let (horizontal_placement, vertical_placement) = section.placement;
        let chunk_x = match horizontal_placement {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (self.map.width - 1) - section.width as i32,
        };
        let chunk_y = match vertical_placement {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => (self.map.height - 1) - section.height as i32,
        };

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }
}
