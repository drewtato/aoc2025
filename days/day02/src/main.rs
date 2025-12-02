use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut sum = 0;
        each_id(input, |id| {
            if !valid(id) {
                sum += id;
            }
        });
        sum
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut sum = 0;
        each_id(input, |id| {
            if !valid2(id) {
                sum += id;
            }
        });
        sum
    }
}

fn valid(id: usize) -> bool {
    let id_length = id.ilog10() + 1;
    if id_length.is_odd() {
        return true;
    }
    // eprintln!("{}", id_length);
    let length = id_length / 2;
    !repeats(length, id)
}

fn valid2(id: usize) -> bool {
    let id_length = id.ilog10() + 1;
    // eprintln!("{}", id_length);
    for length in 1..=id_length / 2 {
        if repeats(length, id) {
            // eprintln!("{}", id);
            return false;
        }
    }
    true
}

fn repeats(length: u32, mut id: usize) -> bool {
    let id_length = id.ilog10() + 1;
    if !id_length.is_multiple_of(length) {
        return false;
    }
    let multiples = id_length / length;
    let divisor = 10usize.pow(length);
    let base = id % divisor;
    id /= divisor;

    for _ in 1..multiples {
        let next = id % divisor;
        if next != base {
            return false;
        }
        id /= divisor;
    }

    true
}

fn each_id(input: &[u8], mut f: impl FnMut(usize)) {
    let mut input = Consume::new(input);
    loop {
        let start = input.int().unwrap();
        assert!(input.byte(b'-'));
        let end = input.int().unwrap();

        for n in start..=end {
            f(n);
        }

        if !input.byte(b',') {
            break;
        }
        input.byte(b'\n');
    }
}
