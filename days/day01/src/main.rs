use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut input = Consume::new(input);
        let mut zeros: usize = 0;
        let mut dial: i32 = 50;

        while !input.is_empty() {
            let direction = match input.consume_byte().unwrap() {
                b'L' => -1,
                b'R' => 1,
                _ => panic!("not L or R"),
            };
            let distance: i32 = input.int().unwrap();
            dial += direction * distance;
            dial = dial.rem_euclid(100);
            if dial == 0 {
                zeros += 1;
            }
            if !input.newline() {
                break;
            }
        }

        zeros
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut input = Consume::new(input);
        let mut zeros: usize = 0;
        let mut dial: i32 = 50;

        while !input.is_empty() {
            let direction = match input.consume_byte().unwrap() {
                b'L' => -1,
                b'R' => 1,
                _ => panic!("not L or R"),
            };
            let distance: i32 = input.int().unwrap();

            for _ in 0..distance {
                dial += direction;
                dial = dial.rem_euclid(100);
                if dial == 0 {
                    zeros += 1;
                }
            }

            if !input.newline() {
                break;
            }
        }

        zeros
    }
}
