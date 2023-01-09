use specs::prelude::*;

use crate::components::*;
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
        ReadStorage<'a, AreaOfEffect>,
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
            aoe,
        ) = data;

        for (entity, use_item) in (&entities, &wants_drink).join() {
            let mut used_item = true;

            // Targeting
            let mut targets = Vec::<Entity>::new();

            if let Some(target) = use_item.target {
                let area_effect = aoe.get(use_item.item);

                if let Some(area_effect) = area_effect {
                    // AoE
                    let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);

                    blast_tiles.retain(|p| {
                        p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                    });

                    for tile_idx in blast_tiles.iter() {
                        let idx = map.xy_idx(tile_idx.x, tile_idx.y);

                        for mob in map.tile_content[idx].iter() {
                            targets.push(*mob);
                        }
                    }
                } else {
                    // Single target in tile
                    let idx = map.xy_idx(target.x, target.y);

                    for mob in map.tile_content[idx].iter() {
                        targets.push(*mob);
                    }
                }
            } else {
                targets.push(*player_entity);
            }

            let heal_item = healings.get(use_item.item);

            if let Some(heal_item) = heal_item {
                for target in targets.iter() {
                    let stats = combat_stats.get_mut(*target);

                    if let Some(stats) = stats {
                        stats.hp = i32::min(stats.max_hp, stats.hp + heal_item.heal_amount);

                        if entity == *player_entity {
                            game_log.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(use_item.item).unwrap().name,
                                heal_item.heal_amount
                            ));
                        }
                    }
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(use_item.item);

            if let Some(damage) = item_damages {
                used_item = false;

                for mob in targets.iter() {
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
