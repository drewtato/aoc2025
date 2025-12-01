#![allow(unused)]

use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        "unimplemented"
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        "unimplemented"
    }
}
