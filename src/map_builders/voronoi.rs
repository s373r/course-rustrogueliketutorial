use crate::map::TileType;
use crate::map_builders::BuilderMap;
use crate::map_builders::InitialMapBuilder;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

pub struct VoronoiCellBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl InitialMapBuilder for VoronoiCellBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder {
    #[allow(dead_code)]
    fn new(distance_algorithm: DistanceAlgorithm) -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            n_seeds: 64,
            distance_algorithm,
        })
    }

    pub fn pythagoras() -> Box<VoronoiCellBuilder> {
        Self::new(DistanceAlgorithm::Pythagoras)
    }

    pub fn manhattan() -> Box<VoronoiCellBuilder> {
        Self::new(DistanceAlgorithm::Manhattan)
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let map_width = build_data.map.width;
        let map_height = build_data.map.height;

        // Make a Voronoi diagram. We'll do this the hard way to learn about the technique!
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, map_width - 1);
            let vy = rng.roll_dice(1, map_height - 1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));

            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32); self.n_seeds];
        let mut voronoi_membership: Vec<i32> = vec![0; map_width as usize * map_height as usize];

        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % map_width;
            let y = i as i32 / map_width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance = match self.distance_algorithm {
                    DistanceAlgorithm::Pythagoras => rltk::DistanceAlg::PythagorasSquared
                        .distance2d(rltk::Point::new(x, y), pos.1),
                    DistanceAlgorithm::Manhattan => {
                        rltk::DistanceAlg::Manhattan.distance2d(rltk::Point::new(x, y), pos.1)
                    }
                    DistanceAlgorithm::Chebyshev => {
                        rltk::DistanceAlg::Chebyshev.distance2d(rltk::Point::new(x, y), pos.1)
                    }
                };

                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1..map_height - 1 {
            for x in 1..map_width - 1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];

                if voronoi_membership[build_data.map.xy_idx(x - 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[build_data.map.xy_idx(x + 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[build_data.map.xy_idx(x, y - 1)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[build_data.map.xy_idx(x, y + 1)] != my_seed {
                    neighbors += 1;
                }

                if neighbors < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }
}
