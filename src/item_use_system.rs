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
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            map,
            entities,
            mut wants_use_item,
            names,
            healings,
            inflict_damage,
            mut combat_stats,
            mut suffer_damage,
            consumables,
            aoe,
            mut confused,
            equippable,
            mut equipped,
            mut backpack,
        ) = data;

        for (entity, use_item) in (&entities, &wants_use_item).join() {
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

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(use_item.item);

            if let Some(Equippable { slot: target_slot }) = item_equippable {
                let target = *targets.first().unwrap();

                // Remove any items the target has in the item's slot
                let mut to_unequip = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == *target_slot {
                        to_unequip.push(item_entity);

                        if target == *player_entity {
                            game_log.entries.push(format!("You unequip {}.", name.name));
                        }
                    }
                }

                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack
                        .insert(*item, InBackpack { owner: target })
                        .expect("Unable to insert backpack entry");
                }

                // Wield the item
                equipped
                    .insert(
                        use_item.item,
                        Equipped {
                            owner: target,
                            slot: *target_slot,
                        },
                    )
                    .expect("Unable to insert equipped component");

                backpack.remove(use_item.item);

                if target == *player_entity {
                    game_log.entries.push(format!(
                        "You equip {}.",
                        names.get(use_item.item).unwrap().name
                    ));
                }
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

            // Can it pass along confusion? Note the use of scopes to escape from the borrow checker!
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(use_item.item);

                if let Some(confusion) = causes_confusion {
                    used_item = false;

                    for mob in targets.iter() {
                        add_confusion.push((*mob, confusion.turns));

                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(use_item.item).unwrap();

                            game_log.entries.push(format!(
                                "You use {} on {}, confusing them.",
                                item_name.name, mob_name.name
                            ));
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert status");
            }

            if used_item {
                let consumable = consumables.get(use_item.item);

                if consumable.is_some() {
                    entities.delete(use_item.item).expect("Delete failed");
                }
            }
        }

        wants_use_item.clear();
    }
}
