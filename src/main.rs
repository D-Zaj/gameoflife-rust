use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::vec;
use std::{thread, time};

const PIXEL_SIZE: usize = 1;
const DELAY_MS: usize = 1;
const RUN_MODE: RunMode = RunMode::Release;

#[allow(dead_code)]
enum RunMode {
    Release,
    Debug,
}

#[derive(Debug, Clone)]
enum CellState {
    Dead = 10,
    Alive = 200,
}

#[derive(Debug, Clone)]
struct Cell {
    state: CellState,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        match self.state {
            CellState::Alive => {
                for _ in 0..PIXEL_SIZE {
                    str.push('⬛')
                }
            }
            CellState::Dead => {
                for _ in 0..PIXEL_SIZE {
                    str.push('⬜')
                }
            }
        }
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
struct Board {
    cells: Vec<Vec<Cell>>,
    next_cells: Vec<Vec<Cell>>,
    rows: usize,
    cols: usize,
}

impl Board {
    fn new(rows: usize, cols: usize) -> Board {
        Board {
            cells: vec![
                vec![
                    Cell {
                        state: CellState::Dead
                    };
                    cols
                ];
                rows
            ],
            next_cells: vec![
                vec![
                    Cell {
                        state: CellState::Dead
                    };
                    cols
                ];
                rows
            ],
            rows,
            cols,
        }
    }

    // Generate Board struct from text file representation
    // (0 for dead cell, 1 for live cell, no spaces)
    pub fn from_file(file_name: &str) -> Board {
        let path = Path::new(&file_name);
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => panic!("Couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).unwrap();

        let rows = file_contents.lines().count();
        let cols = file_contents.lines().nth(1).unwrap().len();

        println!("Generating board of dimensions: {}x{}", rows, cols);
        let mut board = Board::new(rows, cols);
        for (row, line) in file_contents.lines().enumerate() {
            for (col, char) in line.chars().enumerate() {
                let state: CellState = match char {
                    '0' => CellState::Dead,
                    '1' => CellState::Alive,
                    _ => panic!("Invalid char @ line: {}, char: {}", line, char),
                };
                // println!("Setting state for index: {}, {}", row, col);
                board.cells[row][col] = Cell { state };
            }
        }
        board
    }

    // Game loop
    pub fn run_in_terminal(&mut self) {
        let delay = time::Duration::from_millis(DELAY_MS as u64);
        loop {
            println!("\x1bc{}", self);
            self.next_tick();
            thread::sleep(delay);
        }
    }

    fn count_alive_neighbors(&self, row: usize, col: usize) -> usize {
        (-1 as i32..=1 as i32)
            .flat_map(|i| (-1 as i32..=1 as i32).map(move |j| (i, j))) // Produce iterator over pairs
            .filter(|(i, j)| *i != 0 || *j != 0) // Filter out 0,0
            .map(|(i, j)| {
                (
                    (i + (row + self.rows) as i32) % self.rows as i32,
                    (j + (col + self.cols) as i32) % self.cols as i32,
                )
            })
            .filter(|(x, _)| *x >= 0 && *x < self.rows as i32)
            .filter(|(_, y)| *y >= 0 && *y < self.cols as i32)
            .filter(|(x, y)| matches!(self.cells[*x as usize][*y as usize].state, CellState::Alive))
            .count()
    }

    // fn get_alive_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
    //     (-1 as i32..=1 as i32)
    //         .flat_map(|i| (-1 as i32..1 as i32).map(move |j| (i, j))) // Produce iterator over pairs
    //         .filter(|(i, j)| *i != 0 || *j != 0) // Filter out 0,0
    //         .map(|(i, j)| (i + row as i32, j + col as i32))
    //         .filter(|(x, _)| *x >= 0 && *x < self.rows as i32)
    //         .filter(|(_, y)| *y >= 0 && *y < self.cols as i32)
    //         // .filter(|(x, y)| matches!(self.cells[*x as usize][*y as usize].state, CellState::Alive))
    //         .map(|(x, y)| (x as usize, y as usize))
    //         .collect()
    // }

    pub fn next_tick(&mut self) {
        for x in 0..self.rows {
            for y in 0..self.cols {
                let cell = &self.cells[x][y];
                let alive_neighbors_cnt: usize = self.count_alive_neighbors(x, y);
                // println!("Neighbors @ ({}, {}): {:?}", x, y, neighbors);
                match cell.state {
                    CellState::Dead => match alive_neighbors_cnt {
                        3 => self.next_cells[x][y].state = CellState::Alive,
                        _ => self.next_cells[x][y].state = CellState::Dead,
                    },
                    CellState::Alive => match alive_neighbors_cnt {
                        2 | 3 => self.next_cells[x][y].state = CellState::Alive,
                        _ => self.next_cells[x][y].state = CellState::Dead,
                    },
                };
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next_cells);
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = ((usize, usize), &Cell)> {
        self.cells
            .iter()
            .enumerate()
            .flat_map(|(x, row)| row.iter().enumerate().map(move |(y, cell)| ((x, y), cell)))
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();

        // Add col numbers to first line
        let header = (0..self.cols)
            .map(|x| format!("{} ", x % 10))
            .collect::<String>();
        str.push_str("   ");
        str.push_str(&header);
        str.push('\n');

        // Add hoz line to separate from play field
        let box_top = format!(
            "┌{}",
            &std::iter::repeat("─┬").take(self.cols).collect::<String>()
        );
        str.push_str("  ");
        str.push_str(&box_top);
        str.push('\n');

        // Iterate and display the state of each cell
        for (x, row) in self.cells.iter().enumerate() {
            let mut row_str = String::new();
            row_str.push_str(&format!("{:02}│", x));

            for (y, cell) in row.iter().enumerate() {
                // Get the number of live neigbors - used for debugging
                let alive_cnt = self.count_alive_neighbors(x, y);

                let cell_str = match RUN_MODE {
                    RunMode::Debug => match cell.state {
                        CellState::Alive => format!("\x1b[42m{}\x1b[0m ", alive_cnt),
                        CellState::Dead => format!("{}│", alive_cnt),
                    },
                    RunMode::Release => match cell.state {
                        CellState::Alive => format!("{}", cell),
                        CellState::Dead => format!("{}", cell),
                    },
                };
                row_str.push_str(&cell_str);
            }
            row_str.push('\n');
            str.push_str(&row_str);
        }
        write!(f, "{}", str)
    }
}

fn main() {
    let mut test = Board::from_file("/home/dzajac/rust-tests/gameoflife/board.txt");
    // println!("Inital states:");
    // for ((x, y), cell) in test.iter_cells() {
    //     println!("({},{}): {}", x, y, cell);
    // }
    test.run_in_terminal();
}
