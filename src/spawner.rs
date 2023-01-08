use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

use crate::components::{
    BlocksTile, CombatStats, Consumable, Item, Monster, Name, Player, Position, ProvidesHealing,
    Renderable, Viewshed,
};
use crate::map::Map;
use crate::rect::Rect;
use crate::render_order::RenderOrder;

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    // NOTE(DP): debug code
    health_potion(ecs, player_x - 1, player_y - 1);
    health_potion(ecs, player_x - 1, player_y + 1);
    health_potion(ecs, player_x + 1, player_y - 1);
    health_potion(ecs, player_x + 1, player_y + 1);

    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: RenderOrder::Player,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        rng.roll_dice(1, 2)
    };

    match roll {
        1 => orc(ecs, x, y),
        _ => goblin(ecs, x, y),
    }
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc");
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: RenderOrder::Monster,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build();
}

/// Fills a room with stuff!
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _ in 0..num_monsters {
            let mut added = false;

            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * Map::WIDTH) + x;

                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;

            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * Map::WIDTH) + x;

                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        // TODO(DP): extract as a function
        let x = *idx % Map::WIDTH;
        let y = *idx / Map::WIDTH;

        random_monster(ecs, x as i32, y as i32);
    }

    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % Map::WIDTH;
        let y = *idx / Map::WIDTH;

        health_potion(ecs, x as i32, y as i32);
    }
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: RenderOrder::Item,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Consumable {})
        .build();
}
