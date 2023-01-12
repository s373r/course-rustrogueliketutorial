use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::RunState;

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>, // The player
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut hunger_clock,
            player_entity,
            run_state,
            mut inflict_damage,
            mut game_log,
        ) = data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let is_player_entity = entity == *player_entity;
            let proceed = match *run_state {
                RunState::PlayerTurn => is_player_entity,
                RunState::MonsterTurn => !is_player_entity,
                _ => false,
            };

            if !proceed {
                continue;
            }

            clock.duration -= 1;

            if clock.duration.is_positive() {
                continue;
            }

            match clock.state {
                HungerState::WellFed => {
                    clock.state = HungerState::Normal;
                    clock.duration = 200;

                    if is_player_entity {
                        game_log
                            .entries
                            .push("You are no longer well fed.".to_string());
                    }
                }
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.duration = 200;

                    if is_player_entity {
                        game_log.entries.push("You are hungry.".to_string());
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Starving;
                    clock.duration = 200;

                    if is_player_entity {
                        game_log.entries.push("You are starving!".to_string());
                    }
                }
                HungerState::Starving => {
                    // Inflict damage from hunger

                    if is_player_entity {
                        game_log.entries.push(
                            "Your hunger pangs are getting painful! You suffer 1 hp damage."
                                .to_string(),
                        );
                    }

                    SufferDamage::new_damage(&mut inflict_damage, entity, 1);
                }
            }
        }
    }
}
