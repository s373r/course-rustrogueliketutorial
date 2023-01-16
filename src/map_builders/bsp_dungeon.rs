use specs::World;

use crate::components::Position;
use crate::map::Map;
use crate::map_builders::MapBuilder;
use crate::rect::Rect;
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

pub struct BspDungeonBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
}

impl BspDungeonBuilder {
    pub fn new(new_depth: i32) -> BspDungeonBuilder {
        BspDungeonBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new(),
        }
    }
}

impl MapBuilder for BspDungeonBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        // We should do something here
    }

    fn spawn_entities(&self, ecs: &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if !SHOW_MAPGEN_VISUALIZER {
            return;
        }

        let mut snapshot = self.map.clone();
        for v in snapshot.revealed_tiles.iter_mut() {
            *v = true;
        }
        self.history.push(snapshot);
    }
}
