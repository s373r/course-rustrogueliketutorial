mod camera;
mod components;
mod damage_system;
mod game_log;
mod gui;
mod hunger_system;
mod inventory_system;
mod map;
mod map_builders;
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

const SHOW_MAPGEN_VISUALIZER: bool = false;
const SHOW_MAP_AFTER_GENERATION: bool = false;

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
    MapGeneration,
}

pub struct State {
    pub ecs: World,

    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
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
        let current_depth = {
            let map_resource = self.ecs.fetch::<Map>();

            map_resource.depth
        };

        self.generate_world_map(current_depth + 1);

        // Notify the player and give them some health
        let mut game_log = self.ecs.fetch_mut::<GameLog>();

        game_log
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());

        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_entity = *self.ecs.fetch::<Entity>();

        if let Some(player_health) = player_health_store.get_mut(player_entity) {
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

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Build a new map and place the player
        self.generate_world_map(1);
    }

    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        // NOTE(DP): we do not need clear() since there is reassignment later
        // self.mapgen_history.clear();

        let mut rng = self.ecs.write_resource::<RandomNumberGenerator>();
        let mut builder = map_builders::random_builder(new_depth, &mut rng, 80, 50);
        builder.build_map(&mut rng);
        drop(rng);

        self.mapgen_history = builder.build_data.history.clone();

        let player_start = {
            let mut map_resource = self.ecs.write_resource::<Map>();

            *map_resource = builder.build_data.map.clone();

            builder
                .build_data
                .starting_position
                .as_mut()
                .unwrap()
                .clone()
        };

        // Spawn bad guys
        builder.spawn_entities(&mut self.ecs);

        // Place the player and update resources
        let (player_x, player_y) = (player_start.x, player_start.y);
        {
            let mut player_position = self.ecs.write_resource::<Point>();

            *player_position = Point::new(player_x, player_y);
        }

        let player_entity = *self.ecs.fetch::<Entity>();
        let mut position_components = self.ecs.write_storage::<Position>();

        if let Some(player_entity_position) = position_components.get_mut(player_entity) {
            player_entity_position.x = player_x;
            player_entity_position.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();

        if let Some(vs) = viewshed_components.get_mut(player_entity) {
            vs.dirty = true;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        let run_state = *self.ecs.fetch::<RunState>();

        match run_state {
            RunState::MainMenu { .. } | RunState::GameOver { .. } => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx)
            }
        }

        let new_run_state = match run_state {
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
                    gui::ItemMenuResult::NoResponse => run_state,
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
                    gui::ItemMenuResult::NoResponse => run_state,
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
                    gui::ItemMenuResult::NoResponse => run_state,
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
                    gui::ItemMenuResult::NoResponse => run_state,
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
                    gui::GameOverResult::NoSelection => run_state,
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

                for x in 0..map.width {
                    let idx = map.xy_idx(x, row);

                    map.revealed_tiles[idx] = true;
                }

                if row == map.height - 1 {
                    RunState::MonsterTurn
                } else {
                    RunState::MagicMapReveal { row: row + 1 }
                }
            }
            RunState::MapGeneration => {
                let stop_map_generation_visualization =
                    !SHOW_MAPGEN_VISUALIZER || self.mapgen_index >= self.mapgen_history.len();

                if stop_map_generation_visualization {
                    if SHOW_MAP_AFTER_GENERATION {
                        let mut map = self.ecs.fetch_mut::<Map>();

                        map.revealed_tiles.fill(true);

                        camera::render_debug_map(&map, ctx);

                        map.revealed_tiles.fill(false);
                    }

                    self.mapgen_next_state.unwrap()
                } else {
                    ctx.cls();
                    camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);

                    self.mapgen_timer += ctx.frame_time_ms;

                    if self.mapgen_timer > 300.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                    }

                    run_state
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

    // NOTE(DP): added to see generated map
    let mapgen_next_state = Some(if SHOW_MAP_AFTER_GENERATION {
        RunState::MapGeneration
    } else {
        RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }
    });

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state,
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

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
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();

    // Placeholders for Map and player position
    gs.ecs.insert(Map::new(1, 64, 64));
    gs.ecs.insert(Point::new(0, 0));

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(RandomNumberGenerator::new());

    let player_entity = spawner::player(&mut gs.ecs, 0, 0);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());
    gs.ecs.insert(RunState::MapGeneration);

    gs.generate_world_map(1);

    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    rltk::main_loop(context, gs)
}
