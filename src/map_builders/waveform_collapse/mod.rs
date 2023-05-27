mod common;
mod constraints;
mod solver;

use crate::map::Map;
use crate::map_builders::waveform_collapse::common::{patterns_to_constraints, MapChunk};
use crate::map_builders::waveform_collapse::constraints::{build_patterns, render_pattern_to_map};
use crate::map_builders::waveform_collapse::solver::Solver;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    /// Constructor for waveform collapse.
    #[allow(dead_code)]
    pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        const CHUNK_SIZE: i32 = 8;

        build_data.take_snapshot();

        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);

        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);

        build_data.map = Map::new(
            build_data.map.depth,
            build_data.map.width,
            build_data.map.height,
        );

        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }

            build_data.take_snapshot();

            if solver.possible {
                break;
            }

            // If it has hit an impossible condition, try again
        }
    }

    fn render_tile_gallery(
        &mut self,
        patterns: &Vec<MapChunk>,
        chunk_size: i32,
        build_data: &mut BuilderMap,
    ) {
        build_data.map = Map::new(0, build_data.map.width, build_data.map.height);

        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;

        while counter < patterns.len() {
            render_pattern_to_map(&mut build_data.map, &patterns[counter], chunk_size, x, y);

            x += chunk_size + 1;

            if x + chunk_size > build_data.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > build_data.map.height {
                    // Move to the next page
                    build_data.take_snapshot();
                    build_data.map = Map::new(0, build_data.map.width, build_data.map.height);

                    x = 1;
                    y = 1;
                }
            }

            counter += 1;
        }

        build_data.take_snapshot();
    }
}
