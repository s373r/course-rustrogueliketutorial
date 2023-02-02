use rltk::RandomNumberGenerator;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::*;
use crate::map_builders::MapBuilder;
use crate::rect::Rect;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    spawn_list: Vec<SpawnEntity>,
}

impl SimpleMapBuilder {
    pub fn new(new_depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
            spawn_list: Vec::new(),
        }
    }

    pub fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let has_intersect_other_rooms = self
                .rooms
                .iter()
                .any(|another_room| new_room.intersect(another_room));

            if has_intersect_other_rooms {
                continue;
            }

            apply_room(&mut self.map, &new_room);

            self.take_snapshot();

            if !self.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = self.rooms.last().unwrap().center();

                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                }
            }

            self.rooms.push(new_room);

            self.take_snapshot();
        }

        let stairs_position = self.rooms.last().unwrap().center();
        let stairs_idx = self.map.xy_idx(stairs_position.0, stairs_position.1);

        self.map.tiles[stairs_idx] = TileType::DownStairs;

        // Start position
        let (x, y) = self.rooms.first().unwrap().center();

        self.starting_position = Position { x, y };

        // Spawn some entities
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(&self.map, &mut rng, room, self.depth, &mut self.spawn_list);
        }
    }
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self) {
        self.rooms_and_corridors();
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
