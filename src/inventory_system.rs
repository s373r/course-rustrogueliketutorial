use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;
use crate::RunState;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut game_log, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);

            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                game_log.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let drop_position = positions.get(entity).unwrap();

            positions
                .insert(to_drop.item, drop_position.clone())
                .expect("Unable to insert position");

            backpack.remove(to_drop.item);

            if entity == *player_entity {
                game_log.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
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
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
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
            mut particle_builder,
            positions,
            provides_food,
            mut hunger_clocks,
            magic_mapper,
            mut run_state,
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

                        particle_builder.request(
                            tile_idx.x,
                            tile_idx.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('░'),
                            200.0,
                        );
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

            // If its a magic mapper...
            if magic_mapper.get(use_item.item).is_some() {
                used_item = true;

                *run_state = RunState::MagicMapReveal { row: 0 };

                game_log
                    .entries
                    .push("The map is revealed to you!".to_string());
            }

            // It it is edible, eat it!
            if provides_food.get(use_item.item).is_some() {
                used_item = true;

                let target = targets.first().unwrap();

                if let Some(hc) = hunger_clocks.get_mut(*target) {
                    hc.state = HungerState::WellFed;
                    hc.duration = 20;

                    game_log.entries.push(format!(
                        "You eat the {}.",
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

                        if let Some(Position { x, y }) = positions.get(*target) {
                            particle_builder.request(
                                *x,
                                *y,
                                rltk::RGB::named(rltk::GREEN),
                                rltk::RGB::named(rltk::BLACK),
                                rltk::to_cp437('♥'),
                                200.0,
                            );
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

                    if let Some(Position { x, y }) = positions.get(*mob) {
                        particle_builder.request(
                            *x,
                            *y,
                            rltk::RGB::named(rltk::RED),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
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

                        if let Some(Position { x, y }) = positions.get(*mob) {
                            particle_builder.request(
                                *x,
                                *y,
                                rltk::RGB::named(rltk::MAGENTA),
                                rltk::RGB::named(rltk::BLACK),
                                rltk::to_cp437('?'),
                                200.0,
                            );
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
