use crate::components::{Monster, Position, Viewshed, WantsToMelee};
use crate::map::Map;
use crate::RunState;
use rltk::{console, Point};
use specs::prelude::*;

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
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
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

            if path.success && path.steps.len() > 1 {
                let mut idx = map.xy_idx(pos.x, pos.y);

                map.blocked[idx] = false;

                let next_step = path.steps[1] as i32;

                pos.x = next_step % map.width;
                pos.y = next_step / map.width;
                idx = map.xy_idx(pos.x, pos.y);

                map.blocked[idx] = true;

                viewshed.dirty = true;
            }
        }
    }
}
