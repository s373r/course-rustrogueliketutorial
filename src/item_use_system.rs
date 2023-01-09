use specs::prelude::*;

use crate::components::{
    CombatStats, Consumable, InflictsDamage, Name, ProvidesHealing, SufferDamage, WantsToUseItem,
};
use crate::game_log::GameLog;
use crate::map::Map;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Consumable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            map,
            entities,
            mut wants_drink,
            names,
            healings,
            inflict_damage,
            mut combat_stats,
            mut suffer_damage,
            consumables,
        ) = data;

        for (entity, use_item, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            let mut used_item = true;
            let heal_item = healings.get(use_item.item);

            if let Some(heal_item) = heal_item {
                stats.hp = i32::min(stats.max_hp, stats.hp + heal_item.heal_amount);

                if entity == *player_entity {
                    game_log.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        names.get(use_item.item).unwrap().name,
                        heal_item.heal_amount
                    ));
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(use_item.item);

            if let Some(damage) = item_damages {
                used_item = false;

                let target_point = use_item.target.unwrap();
                let idx = map.xy_idx(target_point.x, target_point.y);

                for mob in map.tile_content[idx].iter() {
                    SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);

                    if entity == *player_entity {
                        let mob_name = names.get(*mob).unwrap();
                        let item_name = names.get(use_item.item).unwrap();

                        game_log.entries.push(format!(
                            "You use {} on {}, inflicting {} hp.",
                            item_name.name, mob_name.name, damage.damage
                        ));
                    }

                    used_item = true;
                }
            }

            let consumable = consumables.get(use_item.item);

            if used_item && consumable.is_some() {
                entities.delete(use_item.item).expect("Delete failed");
            }
        }

        wants_drink.clear();
    }
}
