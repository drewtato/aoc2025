use std::cmp::Ordering;

use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let (fresh, input) = fresh_ranges(input);
        let mut count = 0usize;
        each_id(input, |id: u64| {
            if is_fresh(&fresh, id) {
                count += 1;
            }
        });
        count
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let (fresh, _input) = fresh_ranges(input);
        fresh
            .into_iter()
            .map(|[start, end]| (start..=end).count())
            .sum::<usize>()
    }
}

fn each_id(input: &[u8], mut f: impl FnMut(u64)) {
    let mut input = Consume::new(input);
    while !input.is_empty() {
        let id = input.int().unwrap();
        assert!(input.newline());
        f(id);
    }
}

fn is_fresh(fresh: &[[u64; 2]], id: u64) -> bool {
    fresh
        .binary_search_by(|&[start, end]| {
            if id < start {
                Ordering::Greater
            } else if id > end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .is_ok()
}

fn fresh_ranges(input: &[u8]) -> (Vec<[u64; 2]>, &[u8]) {
    let mut input = Consume::new(input);
    let mut ranges = Vec::new();
    while !input.newline() {
        let start = input.int().unwrap();
        assert!(input.byte(b'-'));
        let end: u64 = input.int().unwrap();
        assert!(input.newline());
        ranges.push([start, end]);
    }
    ranges.sort_unstable();
    ranges.dedup_by(|b, a| {
        if a[1] + 1 >= b[0] {
            *a = [a[0], a[1].max(b[1])];
            true
        } else {
            false
        }
    });
    (ranges, input.slice())
}
