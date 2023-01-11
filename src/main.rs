mod components;
mod damage_system;
mod game_log;
mod gui;
mod inventory_system;
mod item_use_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod menu;
mod monster_ai_system;
mod player;
mod rect;
mod render_order;
mod spawner;
mod visibility_system;

use crate::damage_system::DamageSystem;
use crate::game_log::GameLog;
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem};
use crate::item_use_system::ItemUseSystem;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use components::*;
use map::*;
use monster_ai_system::MonsterAI;
use player::*;
use rltk::{GameState, Point, RandomNumberGenerator, Rltk};
use specs::prelude::*;
use visibility_system::*;

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

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_run_state = *self.ecs.fetch::<RunState>();

        match new_run_state {
            RunState::MainMenu { .. } => {}
            _ => {
                draw_map(&self.ecs, ctx);

                let positions = self.ecs.read_storage::<Position>();
                let renderables = self.ecs.read_storage::<Renderable>();
                let map = self.ecs.fetch::<Map>();

                let mut positional_renderables =
                    (&positions, &renderables).join().collect::<Vec<_>>();

                positional_renderables
                    .sort_by(|(_, left), (_, right)| right.render_order.cmp(&left.render_order));

                for (pos, render) in positional_renderables.iter() {
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
                RunState::MonsterTurn
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
                        gui::MainMenuSelection::LoadGame => RunState::PreRun,
                        gui::MainMenuSelection::Quit => std::process::exit(0),
                    },
                }
            }
            RunState::SaveGame => {
                let data = serde_json::to_string(&*self.ecs.fetch::<Map>()).unwrap();

                println!("data: {data}");

                RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
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

    gs.ecs.insert(RandomNumberGenerator::new());

    let map = Map::new_map_rooms_and_corridors();

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
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
