use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut total = 0;
        each(input, |row| {
            total += turn_batteries_on(row, 2);
        });
        total
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut total = 0;
        each(input, |row| {
            total += turn_batteries_on(row, 12);
        });
        total
    }
}

fn turn_batteries_on(row: &[u8], count: usize) -> u64 {
    (0..count)
        .rev()
        .scan(row, |row, c| {
            let (i, &max) = row[0..row.len() - c]
                .iter()
                .enumerate()
                .rev()
                .max_by_key(|&(_, &n)| n)
                .unwrap();
            *row = &row[i + 1..];
            Some(max)
        })
        .fold(0, |acc, joltage| acc * 10 + joltage as u64)
}

fn each(input: &[u8], mut f: impl FnMut(&[u8])) {
    let mut input = Consume::new(input);
    let mut row = Vec::new();
    loop {
        let Some(c) = input.consume_byte() else {
            break;
        };
        match c {
            b'1'..=b'9' => row.push(c - b'0'),
            b'\n' => {
                f(&row);
                row.clear();
            }
            _ => panic!("bad char {c}"),
        }
    }
}
