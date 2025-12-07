use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let start = input.find_byte(b'S').unwrap();
        let width = input[start..].find_byte(b'\n').unwrap() + start + 1;
        let mut beams: ArrayVec<bool, 200> = repeat_n_iter(false, width).collect();
        beams[start] = true;
        let mut splits = 0;
        for row in input.chunks(width).skip(1) {
            for (i, &space) in row.iter().enumerate() {
                match space {
                    b'.' | b'\n' => {}
                    b'^' => {
                        if beams[i] {
                            splits += 1;
                            beams[i] = false;
                            beams[i - 1] = true;
                            beams[i + 1] = true;
                        }
                    }
                    _ => panic!("invalid space"),
                }
            }
        }
        splits
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let start = input.find_byte(b'S').unwrap();
        let width = input[start..].find_byte(b'\n').unwrap() + start + 1;
        let mut beams: ArrayVec<u64, 200> = repeat_n_iter(0, width).collect();
        beams[start] = 1;
        for row in input.chunks(width).skip(1) {
            for (i, &space) in row.iter().enumerate() {
                match space {
                    b'.' | b'\n' => {}
                    b'^' => {
                        let n = replace(&mut beams[i], 0);
                        beams[i - 1] += n;
                        beams[i + 1] += n;
                    }
                    _ => panic!("invalid space"),
                }
            }
        }
        beams.into_iter().sum::<u64>()
    }
}
