use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, SingleActivation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            mut hidden,
            names,
            entities,
            mut log,
            inflicts_damage,
            mut particle_builder,
            mut inflict_damage,
            single_activation,
        ) = data;

        // Iterate the entities that moved and their final position
        let mut remove_entities = Vec::new();

        for (moved_entity, _, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            for entity_in_tile in map.tile_content[idx].iter() {
                if moved_entity == *entity_in_tile {
                    continue;
                }

                // Do not bother to check yourself for being a trap!
                if entry_trigger.get(*entity_in_tile).is_none() {
                    continue;
                }

                // We triggered it
                let name = names.get(*entity_in_tile);

                if let Some(name) = name {
                    log.entries.push(format!("{} triggers!", &name.name));
                }

                // If the trap is damage inflicting, do it
                if let Some(damage) = inflicts_damage.get(*entity_in_tile) {
                    particle_builder.request(
                        pos.x,
                        pos.y,
                        rltk::RGB::named(rltk::ORANGE),
                        rltk::RGB::named(rltk::BLACK),
                        rltk::to_cp437('‼'),
                        200.0,
                    );

                    SufferDamage::new_damage(&mut inflict_damage, moved_entity, damage.damage);
                }

                // If it is single activation, it needs to be removed
                if single_activation.get(*entity_in_tile).is_some() {
                    remove_entities.push(*entity_in_tile);
                }

                hidden.remove(*entity_in_tile); // The trap is no longer hidden
            }
        }

        // Remove any single activation traps
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
