use rltk::Point;
use specs::prelude::*;

use crate::components::*;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;
use crate::RunState;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, EntityMoved>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            run_state,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
            mut particle_builder,
            mut entity_moved,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            let mut can_act = true;
            let is_confused = confused.get_mut(entity);

            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;

                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }

                can_act = false;

                particle_builder.request(
                    pos.x,
                    pos.y,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0,
                );
            }

            if !can_act {
                continue;
            }

            if !viewshed.visible_tiles.contains(&*player_pos) {
                continue;
            }

            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);

            if distance < 1.5 {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *player_entity,
                        },
                    )
                    .expect("Unable to insert attack");

                continue;
            }

            // Path to the player
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(player_pos.x, player_pos.y) as i32,
                &*map,
            );

            // TODO(DP): monsters are stuck near corridor exits (do not move diagonally)
            if path.success && path.steps.len() > 1 {
                let mut idx = map.xy_idx(pos.x, pos.y);

                map.blocked[idx] = false;

                let next_step = path.steps[1] as i32;

                pos.x = next_step % map.width;
                pos.y = next_step / map.width;

                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");

                idx = map.xy_idx(pos.x, pos.y);

                map.blocked[idx] = true;

                viewshed.dirty = true;
            }
        }
    }
}
