use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut splits = 0usize;
        solve(input, true, |i, beams| {
            if beams[i] {
                splits += 1;
                beams[i] = false;
                beams[i - 1] = true;
                beams[i + 1] = true;
            }
        });
        splits
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        solve(input, 1, |i, beams| {
            let n = replace(&mut beams[i], 0);
            beams[i - 1] += n;
            beams[i + 1] += n;
        })
        .into_iter()
        .sum::<u64>()
    }
}

fn solve<T: Default>(
    input: &[u8],
    init: T,
    mut f: impl FnMut(usize, &mut [T]),
) -> ArrayVec<T, 200> {
    let start = input.find_byte(b'S').unwrap();
    let width = input[start..].find_byte(b'\n').unwrap() + start + 1;
    let mut beams: ArrayVec<_, 200> = repeat_with_iter(T::default).take(width).collect();
    beams[start] = init;
    for row in input.chunks(width).skip(1) {
        for (i, &space) in row.iter().enumerate().take(width - 1) {
            match space {
                b'.' => {}
                b'^' => f(i, &mut beams),
                _ => panic!("invalid space"),
            }
        }
    }
    beams
}
