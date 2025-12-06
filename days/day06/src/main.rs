use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        solve(input, |lines, len| {
            lines
                .iter_mut()
                .map(|line| {
                    let next = &line.slice()[len + 1..];
                    line.whitespace();
                    let n: u64 = line.int().unwrap();
                    *line = Consume::new(next);
                    n
                })
                .collect()
        })
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        solve(input, |lines, n| {
            let ns = (0..n)
                .map(|_| {
                    lines
                        .iter_mut()
                        .map(|l| l.consume_byte().unwrap())
                        .fold(0, |acc, b| {
                            if b == b' ' {
                                return acc;
                            }
                            acc * 10 + (b - b'0') as u64
                        })
                })
                .collect();
            for l in &mut *lines {
                l.consume_byte().unwrap();
            }
            ns
        })
    }
}

fn solve(input: &[u8], mut f: impl FnMut(&mut [Consume<'_>], usize) -> ArrayVec<u64, 5>) -> u64 {
    let line_length = input.find_byte(b'\n').unwrap() + 1;
    let line_count = input.len() / line_length;
    let mut lines: ArrayVec<_, 5> = (0..line_count)
        .map(|y| {
            let l = &input[y * line_length..(y + 1) * line_length];
            Consume::new(l)
        })
        .collect();
    let mut operators = lines.pop().unwrap();
    let mut sum = 0u64;

    while !operators.is_empty() {
        let op = operators.consume_byte().unwrap();
        let len = operators.whitespace().len();

        let iter = f(&mut lines, len).into_iter();

        sum += match op {
            b'+' => iter.sum::<u64>(),
            b'*' => iter.product::<u64>(),
            _ => panic!("invalid symbol {}", op as char),
        };
    }

    sum
}
