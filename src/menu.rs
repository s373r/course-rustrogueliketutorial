use rltk::{Rltk, VirtualKeyCode, RGB};

use crate::gui::{MainMenuResult, MainMenuSelection};
use crate::rex_assets::RexAssets;
use crate::{saveload_system, RunState, State};

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let assets = gs.ecs.fetch::<RexAssets>();

    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(
        24,
        18,
        31,
        10,
        RGB::named(rltk::WHEAT),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color_centered(
        20,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Rust Roguelike Tutorial",
    );
    ctx.print_color_centered(
        21,
        RGB::named(rltk::CYAN),
        RGB::named(rltk::BLACK),
        "by Herbert Wolverson",
    );
    ctx.print_color_centered(
        22,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        "Use Up/Down Arrows and Enter",
    );

    let run_state = *gs.ecs.fetch::<RunState>();
    let save_exists = saveload_system::does_save_exist();
    let mut y = 24;

    match run_state {
        RunState::MainMenu {
            menu_selection: selection,
        } => {
            if selection == MainMenuSelection::NewGame {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::MAGENTA),
                    RGB::named(rltk::BLACK),
                    "Begin New Game",
                );
            } else {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    "Begin New Game",
                );
            }
            y += 1;

            if save_exists {
                if selection == MainMenuSelection::LoadGame {
                    ctx.print_color_centered(
                        y,
                        RGB::named(rltk::MAGENTA),
                        RGB::named(rltk::BLACK),
                        "Load Game",
                    );
                } else {
                    ctx.print_color_centered(
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::BLACK),
                        "Load Game",
                    );
                }
                y += 1;
            }

            if selection == MainMenuSelection::Quit {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::MAGENTA),
                    RGB::named(rltk::BLACK),
                    "Quit",
                );
            } else {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    "Quit",
                );
            }

            match ctx.key {
                None => MainMenuResult::NoSelection {
                    selected: selection,
                },
                Some(key) => match key {
                    VirtualKeyCode::Escape => MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    },
                    VirtualKeyCode::Up => {
                        let mut new_selection = match selection {
                            MainMenuSelection::NewGame => MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => MainMenuSelection::LoadGame,
                        };

                        if new_selection == MainMenuSelection::LoadGame && !save_exists {
                            new_selection = MainMenuSelection::NewGame;
                        }

                        MainMenuResult::NoSelection {
                            selected: new_selection,
                        }
                    }
                    VirtualKeyCode::Down => {
                        let mut new_selection = match selection {
                            MainMenuSelection::NewGame => MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => MainMenuSelection::Quit,
                            MainMenuSelection::Quit => MainMenuSelection::NewGame,
                        };

                        if new_selection == MainMenuSelection::LoadGame && !save_exists {
                            new_selection = MainMenuSelection::Quit;
                        }

                        MainMenuResult::NoSelection {
                            selected: new_selection,
                        }
                    }
                    VirtualKeyCode::Return => MainMenuResult::Selected {
                        selected: selection,
                    },
                    _ => MainMenuResult::NoSelection {
                        selected: selection,
                    },
                },
            }
        }
        _ => MainMenuResult::NoSelection {
            selected: MainMenuSelection::NewGame,
        },
    }
}
