use specs::prelude::*;

use crate::components::{CombatStats, Consumable, Name, ProvidesHealing, WantsToDrinkPotion};
use crate::game_log::GameLog;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            entities,
            mut wants_drink,
            names,
            healings,
            mut combat_stats,
            consumables,
        ) = data;

        for (entity, use_item, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            let heal_item = healings.get(use_item.potion);

            if let Some(heal_item) = heal_item {
                stats.hp = i32::min(stats.max_hp, stats.hp + heal_item.heal_amount);

                if entity == *player_entity {
                    game_log.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        names.get(use_item.potion).unwrap().name,
                        heal_item.heal_amount
                    ));
                }
            }

            let consumable = consumables.get(use_item.potion);

            if consumable.is_some() {
                entities.delete(use_item.potion).expect("Delete failed");
            }
        }

        wants_drink.clear();
    }
}
