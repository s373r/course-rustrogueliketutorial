use crate::map::{Map, TileType};
use crate::map_builders::BuilderMap;
use crate::map_builders::InitialMapBuilder;

pub struct MazeBuilder {}

impl InitialMapBuilder for MazeBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MazeBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<MazeBuilder> {
        Box::new(MazeBuilder {})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Maze gen
        let mut maze = Grid::new(
            (build_data.map.width / 2) - 2,
            (build_data.map.height / 2) - 2,
            rng,
        );

        maze.generate_maze(build_data);
    }
}

/* Maze code taken under MIT from https://github.com/cyucelen/mazeGenerator/ */

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell {
            row,
            column,
            walls: [true, true, true, true],
            visited: false,
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut rltk::RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: i32, height: i32, rng: &mut rltk::RandomNumberGenerator) -> Grid {
        let mut grid = Grid {
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng,
        };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column));
            }
        }

        grid
    }

    // TODO(DP): use Option<T>
    fn calculate_index(&self, row: i32, column: i32) -> i32 {
        if row < 0 || column < 0 || column > self.width - 1 || row > self.height - 1 {
            -1
        } else {
            column + (row * self.width)
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let current_row = self.cells[self.current].row;
        let current_column = self.cells[self.current].column;

        let neighbor_indices: [i32; 4] = [
            self.calculate_index(current_row - 1, current_column),
            self.calculate_index(current_row, current_column + 1),
            self.calculate_index(current_row + 1, current_column),
            self.calculate_index(current_row, current_column - 1),
        ];

        let mut neighbors = Vec::new();

        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();

        if neighbors.is_empty() {
            return None;
        }

        if neighbors.len() == 1 {
            Some(neighbors[0])
        } else {
            let random_neighbor_idx = self.rng.roll_dice(1, neighbors.len() as i32) - 1;

            Some(neighbors[random_neighbor_idx as usize])
        }
    }

    fn generate_maze(&mut self, build_data: &mut BuilderMap) {
        // NOTE(DP): for generation speed-up
        let mut i = 0;

        loop {
            self.cells[self.current].visited = true;

            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.cells[next].visited = true;

                    self.backtrace.push(self.current);
                    //   __lower_part__      __higher_part_
                    //   /            \      /            \
                    // --------cell1------ | cell2-----------
                    let (lower_part, higher_part) =
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];

                    cell1.remove_walls(cell2);

                    self.current = next;
                }
                None => {
                    if self.backtrace.is_empty() {
                        // NOTE(DP): to save the latest state
                        self.copy_to_map(&mut build_data.map);
                        build_data.take_snapshot();

                        break;
                    }

                    self.current = self.backtrace[0];
                    self.backtrace.remove(0);
                }
            }

            if i % 50 == 0 {
                self.copy_to_map(&mut build_data.map);
                build_data.take_snapshot();
            }

            i += 1;
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        // Clear the map
        for i in map.tiles.iter_mut() {
            *i = TileType::Wall;
        }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;

            if !cell.walls[TOP] {
                map.tiles[idx - map.width as usize] = TileType::Floor
            }
            if !cell.walls[RIGHT] {
                map.tiles[idx + 1] = TileType::Floor
            }
            if !cell.walls[BOTTOM] {
                map.tiles[idx + map.width as usize] = TileType::Floor
            }
            if !cell.walls[LEFT] {
                map.tiles[idx - 1] = TileType::Floor
            }
        }
    }
}
