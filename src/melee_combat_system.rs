use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::particle_system::ParticleBuilder;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp <= 0 {
                continue;
            }

            let target_stats = combat_stats.get(wants_melee.target).unwrap();

            if target_stats.hp <= 0 {
                continue;
            }

            let mut offensive_bonus = 0;

            for (_, power_bonus, equipped_by) in (&entities, &melee_power_bonuses, &equipped).join()
            {
                if equipped_by.owner == entity {
                    offensive_bonus += power_bonus.power;
                }
            }

            if let Some(HungerClock { state, .. }) = hunger_clock.get(entity) {
                if *state == HungerState::WellFed {
                    offensive_bonus += 1;
                }
            }

            let target_name = names.get(wants_melee.target).unwrap();

            let mut defensive_bonus = 0;

            for (_, defense_bonus, equipped_by) in (&entities, &defense_bonuses, &equipped).join() {
                if equipped_by.owner == wants_melee.target {
                    defensive_bonus += defense_bonus.defense;
                }
            }

            if let Some(Position { x, y }) = positions.get(wants_melee.target) {
                particle_builder.request(
                    *x,
                    *y,
                    rltk::RGB::named(rltk::ORANGE),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('‼'),
                    200.0,
                );
            }

            let damage = i32::max(
                0,
                (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus),
            );

            if damage == 0 {
                log.entries.push(format!(
                    "{} is unable to hurt {}",
                    &name.name, &target_name.name
                ));
            } else {
                log.entries.push(format!(
                    "{} hits {}, for {} hp.",
                    &name.name, &target_name.name, damage
                ));
                SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
            }
        }

        wants_melee.clear();
    }
}
