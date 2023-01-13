mod components;
mod damage_system;
mod game_log;
mod gui;
mod hunger_system;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod menu;
mod monster_ai_system;
mod particle_system;
mod player;
mod random_table;
mod rect;
mod render_order;
mod rex_assets;
mod saveload_system;
mod spawner;
mod trigger_system;
mod visibility_system;

use rltk::{GameState, Point, RandomNumberGenerator, Rltk};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use crate::components::*;
use crate::damage_system::DamageSystem;
use crate::game_log::GameLog;
use crate::inventory_system::{
    ItemCollectionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem,
};
use crate::map::*;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::*;
use crate::visibility_system::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut triggers = trigger_system::TriggerSystem {};
        triggers.run_now(&self.ecs);

        let mut map_index = MapIndexingSystem {};
        map_index.run_now(&self.ecs);

        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);

        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);

        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

        let mut use_items = ItemUseSystem {};
        use_items.run_now(&self.ecs);

        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);

        let mut item_remove = ItemRemoveSystem {};
        item_remove.run_now(&self.ecs);

        let mut hunger = hunger_system::HungerSystem {};
        hunger.run_now(&self.ecs);

        let mut particles = particle_system::ParticleSpawnSystem {};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete = Vec::new();

        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let is_entity_player = player.get(entity).is_some();

            if is_entity_player {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let entity_backpack = backpack.get(entity);

            if let Some(InBackpack { owner }) = entity_backpack {
                if *owner == *player_entity {
                    should_delete = false;
                }
            }

            if let Some(Equipped { owner, .. }) = equipped.get(entity) {
                if *owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();

        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        // Build a new map and place the player
        let map = {
            let mut map_resource = self.ecs.write_resource::<Map>();
            let current_depth = map_resource.depth;

            *map_resource = Map::new_map_rooms_and_corridors(current_depth + 1);

            map_resource.clone()
        };

        // Spawn bad guys
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, map.depth);
        }

        // Place the player and update resources
        let (player_x, player_y) = map.rooms[0].center();
        let mut player_position = self.ecs.write_resource::<Point>();

        *player_position = Point::new(player_x, player_y);

        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);

        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);

        if let Some(vs) = vs {
            vs.dirty = true;
        }

        // Notify the player and give them some health
        let mut game_log = self.ecs.fetch_mut::<GameLog>();

        game_log
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());

        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);

        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Build a new map and place the player
        let map = {
            let mut map_resource = self.ecs.write_resource::<Map>();

            *map_resource = Map::new_map_rooms_and_corridors(1);

            map_resource.clone()
        };

        // Spawn bad guys
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, 1);
        }

        // Place the player and update resources
        let (player_x, player_y) = map.rooms.first().unwrap().center();
        let player_entity = spawner::player(&mut self.ecs, player_x, player_y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();

        *player_entity_writer = player_entity;

        let player_pos_comp = position_components.get_mut(player_entity);

        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        let mut new_run_state = *self.ecs.fetch::<RunState>();

        match new_run_state {
            RunState::MainMenu { .. } | RunState::GameOver { .. } => {}
            _ => {
                draw_map(&self.ecs, ctx);

                let positions = self.ecs.read_storage::<Position>();
                let renderables = self.ecs.read_storage::<Renderable>();
                let hidden = self.ecs.read_storage::<Hidden>();
                let map = self.ecs.fetch::<Map>();

                let mut positional_renderables = (&positions, &renderables, !&hidden)
                    .join()
                    .collect::<Vec<_>>();

                positional_renderables.sort_by(|(_, left, _), (_, right, _)| {
                    right.render_order.cmp(&left.render_order)
                });

                for (pos, render, _) in positional_renderables.iter() {
                    let idx = map.xy_idx(pos.x, pos.y);

                    if map.visible_tiles[idx] {
                        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                    }
                }

                gui::draw_ui(&self.ecs, ctx)
            }
        }

        new_run_state = match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::AwaitingInput => player_input(self, ctx),
            RunState::PlayerTurn => {
                self.run_systems();

                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => RunState::MagicMapReveal { row: 0 },
                    _ => RunState::MonsterTurn,
                }
            }
            RunState::MonsterTurn => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => new_run_state,
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let ranged_storage = self.ecs.read_storage::<Ranged>();
                        let ranged = ranged_storage.get(item_entity);

                        if let Some(ranged) = ranged {
                            RunState::ShowTargeting {
                                range: ranged.range,
                                item: item_entity,
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();

                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");

                            RunState::PlayerTurn
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => new_run_state,
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();

                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");

                        RunState::PlayerTurn
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let (item_menu_result, optional_point) = gui::ranged_target(self, ctx, range);

                match item_menu_result {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => new_run_state,
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();

                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: optional_point,
                                },
                            )
                            .expect("Unable to insert intent");

                        RunState::PlayerTurn
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = menu::main_menu(self, ctx);

                match result {
                    gui::MainMenuResult::NoSelection { selected } => RunState::MainMenu {
                        menu_selection: selected,
                    },
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            saveload_system::delete_save();
                            RunState::AwaitingInput
                        }
                        gui::MainMenuSelection::Quit => std::process::exit(0),
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);

                RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                }
            }
            RunState::NextLevel => {
                self.goto_next_level();

                RunState::PreRun
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => new_run_state,
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();

                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");

                        RunState::PlayerTurn
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => new_run_state,
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();

                        RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        }
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();

                for x in 0..Map::WIDTH {
                    let idx = map.xy_idx(x as i32, row);

                    map.revealed_tiles[idx] = true;
                }

                if row as usize == Map::HEIGHT - 1 {
                    RunState::MonsterTurn
                } else {
                    RunState::MagicMapReveal { row: row + 1 }
                }
            }
        };

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();

            *run_writer = new_run_state;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    // NOTE(DP): disable the scan lines effect
    // context.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(RandomNumberGenerator::new());
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    let initial_map_depth = 1;
    let map = Map::new_map_rooms_and_corridors(initial_map_depth);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, initial_map_depth);
    }

    let (player_x, player_y) = map.rooms.first().unwrap().center();

    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(map);

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });
    gs.ecs.insert(RandomNumberGenerator::new());

    rltk::main_loop(context, gs)
}
