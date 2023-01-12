use rltk::{Rltk, RGB};
use specs::prelude::*;

use crate::components::*;
use crate::render_order::RenderOrder;

pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        // Age out particles
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }
    }

    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die");
    }
}

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        });
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles, mut particle_builder) = data;

        for particle_request in particle_builder.requests.iter() {
            let particle_entity = entities.create();

            positions
                .insert(
                    particle_entity,
                    Position {
                        x: particle_request.x,
                        y: particle_request.y,
                    },
                )
                .expect("Unable to insert position");

            renderables
                .insert(
                    particle_entity,
                    Renderable {
                        fg: particle_request.fg,
                        bg: particle_request.bg,
                        glyph: particle_request.glyph,
                        render_order: RenderOrder::Particle,
                    },
                )
                .expect("Unable to insert renderable");

            particles
                .insert(
                    particle_entity,
                    ParticleLifetime {
                        lifetime_ms: particle_request.lifetime,
                    },
                )
                .expect("Unable to insert lifetime");
        }

        particle_builder.requests.clear();
    }
}
