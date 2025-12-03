#![allow(unused)]

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

fn turn_batteries_on(mut row: &[u8], count: usize) -> u64 {
    let mut joltage = 0;

    for c in (0..count).rev() {
        let (i, &max) = row[0..row.len() - c]
            .iter()
            .enumerate()
            .rev()
            .max_by_key(|&(_, &n)| n)
            .unwrap();

        joltage *= 10;
        joltage += max as u64;
        row = &row[i + 1..];
    }

    joltage
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
