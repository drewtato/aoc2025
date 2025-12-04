use std::io::Write;

use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let grid = make_grid(input);
        let width = grid[0].len();
        let height = grid.len();
        let mut accessible = 0;
        for y in 0..height {
            for x in 0..width {
                if !*grid_get(&grid, [y, x]).unwrap() {
                    continue;
                }
                let mut adjacent = 0;
                neighbors(&grid, [y, x], |b| {
                    if b {
                        adjacent += 1
                    }
                });
                if adjacent < 4 {
                    accessible += 1;
                }
            }
        }
        accessible
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut grid = make_grid(input);
        let width = grid[0].len();
        let height = grid.len();
        let mut accessible = Vec::<[usize; 2]>::new();
        let mut removed = 0;

        loop {
            // print_grid(&grid);
            for y in 0..height {
                for x in 0..width {
                    if !*grid_get(&grid, [y, x]).unwrap() {
                        continue;
                    }
                    let mut adjacent = 0;
                    neighbors(&grid, [y, x], |b| {
                        if b {
                            adjacent += 1
                        }
                    });
                    if adjacent < 4 {
                        accessible.push([y, x]);
                    }
                }
            }

            if accessible.is_empty() {
                break;
            }
            removed += accessible.len();
            for c in accessible.drain(..) {
                *grid_get_mut(&mut grid, c).unwrap() = false;
            }
        }

        removed
    }
}

fn make_grid(input: &[u8]) -> Vec<Vec<bool>> {
    let mut input = Consume::new(input);
    let mut grid = Vec::new();
    while !input.is_empty() {
        let mut line = input.next_newline();
        line.split_off_last();
        let row = line.iter().map(|&b| b == b'@').collect();
        grid.push(row);
    }
    grid
}

fn neighbors(grid: &[Vec<bool>], coord: [usize; 2], mut f: impl FnMut(bool)) {
    let [y, x] = coord;
    for [dy, dx] in [
        [-1, -1],
        [-1, 0],
        [-1, 1],
        [0, -1],
        [0, 1],
        [1, -1],
        [1, 0],
        [1, 1],
    ] {
        (|| {
            let &b = grid_get(grid, [y.checked_add_signed(dy)?, x.checked_add_signed(dx)?])?;
            f(b);
            Some(())
        })();
    }
}

fn grid_get<T>(grid: &[Vec<T>], coord: [usize; 2]) -> Option<&T> {
    let row = grid.get(coord[0])?;
    let b = row.get(coord[1])?;
    Some(b)
}

fn grid_get_mut<T>(grid: &mut [Vec<T>], coord: [usize; 2]) -> Option<&mut T> {
    let row = grid.get_mut(coord[0])?;
    let b = row.get_mut(coord[1])?;
    Some(b)
}

#[allow(dead_code)]
fn print_grid(grid: &[Vec<bool>]) {
    let mut stderr = std::io::BufWriter::new(std::io::stderr().lock());
    for row in grid {
        for &c in row {
            write!(stderr, "{}", if c { '@' } else { ' ' }).unwrap();
        }
        writeln!(stderr).unwrap();
    }
    writeln!(stderr).unwrap();
    stderr.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
}
