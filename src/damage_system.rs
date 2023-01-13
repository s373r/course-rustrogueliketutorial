use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;
use crate::RunState;

use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities) = data;

        for (entity, mut stats, damage) in (&entities, &mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();

            if let Some(Position { x, y }) = positions.get(entity) {
                let idx = map.xy_idx(*x, *y);

                map.bloodstains.insert(idx);
            }
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let entities = ecs.entities();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp > 0 {
                continue;
            }

            match players.get(entity) {
                None => {
                    let victim_name = names.get(entity).unwrap();

                    log.entries.push(format!("{} is dead", &victim_name.name));

                    dead.push(entity);
                }
                Some(_) => {
                    let mut run_state = ecs.write_resource::<RunState>();
                    *run_state = RunState::GameOver;
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
