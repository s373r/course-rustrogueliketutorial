use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement {})
    }

    fn doors(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(halls) = &build_data.corridors {
            for hall in halls.clone() {
                // We aren't interested in tiny corridors
                if hall.len() < 3 {
                    continue;
                }

                let door_index = hall[0];

                if !self.door_possible(build_data, door_index) {
                    build_data.spawn_list.push((door_index, "Door".to_string()));
                }
            }
        } else {
            // There are no corridors - scan for possible places
            let tiles = build_data.map.tiles.clone();

            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor
                    && self.door_possible(build_data, i)
                    && rng.roll_dice(1, 3) == 1
                {
                    build_data.spawn_list.push((i, "Door".to_string()));
                }
            }
        }
    }

    fn door_possible(&self, build_data: &mut BuilderMap, idx: usize) -> bool {
        for (spawn_entity_map_idx, _) in build_data.spawn_list.iter() {
            if *spawn_entity_map_idx == idx {
                return false;
            }
        }

        let x = idx % build_data.map.width as usize;
        let y = idx / build_data.map.width as usize;

        // Check for east-west door possibility
        if build_data.map.tiles[idx] == TileType::Floor
            && (x > 1 && build_data.map.tiles[idx - 1] == TileType::Floor)
            && (x < (build_data.map.width - 2) as usize
                && build_data.map.tiles[idx + 1] == TileType::Floor)
            && (y > 1
                && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall)
            && (y < (build_data.map.height - 2) as usize
                && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall)
        {
            return true;
        }

        // Check for north-south door possibility
        if build_data.map.tiles[idx] == TileType::Floor
            && (x > 1 && build_data.map.tiles[idx - 1] == TileType::Wall)
            && (x < (build_data.map.width - 2) as usize
                && build_data.map.tiles[idx + 1] == TileType::Wall)
            && (y > 1
                && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Floor)
            && (y < (build_data.map.height - 2) as usize
                && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Floor)
        {
            return true;
        }

        false
    }
}
