pub mod prefab_levels;
pub mod prefab_rooms;
pub mod prefab_sections;

use std::collections::HashSet;

use crate::components::Position;
use crate::map::TileType;
use crate::map_builders::MetaMapBuilder;
use crate::map_builders::{BuilderMap, InitialMapBuilder};

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
    RoomVaults,
}

pub struct PrefabBuilder {
    mode: PrefabMode,
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    fn new(mode: PrefabMode) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder { mode })
    }

    #[allow(dead_code)]
    pub fn rex_level(template: &'static str) -> Box<PrefabBuilder> {
        Self::new(PrefabMode::RexLevel { template })
    }

    #[allow(dead_code)]
    pub fn constant(level: prefab_levels::PrefabLevel) -> Box<PrefabBuilder> {
        Self::new(PrefabMode::Constant { level })
    }

    #[allow(dead_code)]
    pub fn sectional(section: prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        Self::new(PrefabMode::Sectional { section })
    }

    #[allow(dead_code)]
    pub fn vaults() -> Box<PrefabBuilder> {
        Self::new(PrefabMode::RoomVaults)
    }

    fn char_to_map(&mut self, ch: char, idx: usize, build_data: &mut BuilderMap) {
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.starting_position = Option::from(Position { x, y });
            }
            '>' => build_data.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data
                    .spawn_list
                    .push((idx, "Health Potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str, build_data: &mut BuilderMap) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();

                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);

                        self.char_to_map(cell.ch as u8 as char, idx, build_data);
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
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data: &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);

                    self.char_to_map(string_vec[i], idx, build_data);
                }

                i += 1;
            }
        }
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data),
        }

        build_data.take_snapshot();
    }

    fn apply_previous_iteration<F>(
        &mut self,
        mut filter: F,
        _rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) where
        F: FnMut(i32, i32) -> bool,
    {
        let width = build_data.map.width;

        build_data.spawn_list.retain(|(idx, _name)| {
            // TODO(DP): use idx -> (x, y) function
            let x = *idx as i32 % width;
            let y = *idx as i32 / width;

            filter(x, y)
        });

        build_data.take_snapshot();
    }

    fn apply_sectional(
        &mut self,
        section: &prefab_sections::PrefabSection,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        use prefab_sections::*;

        let map_width = build_data.map.width;
        let map_height = build_data.map.height;

        // Place the new section
        let (horizontal_placement, vertical_placement) = section.placement;
        let chunk_x = match horizontal_placement {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (map_width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (map_width - 1) - section.width as i32,
        };
        let chunk_y = match vertical_placement {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (map_height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => (map_height - 1) - section.height as i32,
        };

        // Build the map
        self.apply_previous_iteration(
            |x, y| {
                x < chunk_x
                    || x > (chunk_x + section.width as i32)
                    || y < chunk_y
                    || y > (chunk_y + section.height as i32)
            },
            rng,
            build_data,
        );

        build_data.take_snapshot();

        let string_vec = Self::read_ascii_to_vec(section.template);
        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < map_width as usize && ty < map_height as usize {
                    let idx = build_data
                        .map
                        .xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);

                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_room_vaults(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        use prefab_rooms::*;

        let map_depth = build_data.map.depth;
        let map_width = build_data.map.width;
        let map_height = build_data.map.height;

        // Apply the previous builder, and keep all entities it spawns (for now)
        self.apply_previous_iteration(|_, _| true, rng, build_data);

        // Do we want a vault at all?
        let vault_roll = rng.roll_dice(1, 6) + map_depth;
        if vault_roll < 4 {
            return;
        }

        // Note that this is a place-holder and will be moved out of this function
        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, CHECKERBOARD, SILLY_SMILE];

        // Filter the vault list down to ones that are applicable to the current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| map_depth >= v.first_depth && map_depth <= v.last_depth)
            .collect();

        if possible_vaults.is_empty() {
            return;
        } // Bail out if there's nothing to build

        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles: HashSet<usize> = HashSet::new();

        for _ in 0..n_vaults {
            let vault_index = if possible_vaults.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize
            };

            let vault = possible_vaults[vault_index];
            // We'll make a list of places in which the vault could fit
            let mut vault_positions: Vec<Position> = Vec::new();

            let mut idx = 0usize;
            loop {
                // TODO(DP): use idx -> (x, y) function
                let x = (idx % map_width as usize) as i32;
                let y = (idx / map_width as usize) as i32;

                // Check that we won't overflow the map
                if x > 1
                    && (x + vault.width as i32) < map_width - 2
                    && y > 1
                    && (y + vault.height as i32) < map_height - 2
                {
                    let mut possible = true;
                    for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            let idx = build_data.map.xy_idx(tx + x, ty + y);

                            if build_data.map.tiles[idx] != TileType::Floor {
                                possible = false;
                            }
                            if used_tiles.contains(&idx) {
                                possible = false;
                            }
                        }
                    }

                    if possible {
                        vault_positions.push(Position { x, y });
                        break;
                    }
                }

                idx += 1;
                if idx >= build_data.map.tiles.len() - 1 {
                    break;
                }
            }

            if !vault_positions.is_empty() {
                let pos_idx = if vault_positions.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize
                };
                let pos = &vault_positions[pos_idx];

                let chunk_x = pos.x;
                let chunk_y = pos.y;

                build_data.spawn_list.retain(|(map_idx, _)| {
                    let map_idx = *map_idx as i32;
                    let x = map_idx % map_width;
                    let y = map_idx / map_height;

                    x < chunk_x
                        || x > chunk_x + vault.width as i32
                        || y < chunk_y
                        || y > chunk_y + vault.height as i32
                });

                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        let idx = build_data
                            .map
                            .xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);

                        self.char_to_map(string_vec[i], idx, build_data);

                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                build_data.take_snapshot();

                possible_vaults.remove(vault_index);
            }
        }
    }
}
