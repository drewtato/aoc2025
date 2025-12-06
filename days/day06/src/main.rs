use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut input = Consume::new(input);
        let mut numbers = Vec::new();

        while !(input.slice()[0] == (b'+') || input.slice()[0] == b'*') {
            let mut row = Vec::new();
            while !input.newline() {
                while input.byte(b' ') {}
                row.push(input.int::<u64>().unwrap());
                while input.byte(b' ') {}
            }
            numbers.push(row);
        }

        let mut sum = 0;
        let mut i = 0;
        while !input.newline() {
            let symbol = input.consume_byte().unwrap();
            while input.byte(b' ') {}

            sum += match symbol {
                b'+' => numbers.iter().map(|row| row[i]).sum::<u64>(),
                b'*' => numbers.iter().map(|row| row[i]).product::<u64>(),
                _ => panic!("bad symbol"),
            };
            i += 1;
        }

        sum
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let line_length = input.find_byte(b'\n').unwrap() + 1;
        let line_count = input.len() / line_length;

        let last_line = &input[(line_count - 1) * line_length..];
        let mut sum = 0u64;
        let mut col = 0;
        while col < line_length {
            let operator = last_line[col];

            // + 1 for operator and - 1 for empty column
            let numbers = last_line[col + 1..]
                .iter()
                .take_while(|&&b| b == b' ' || b == b'\n')
                .count();

            let total = match operator {
                b'+' => operate(
                    0,
                    |a, b| a + b,
                    line_length,
                    col,
                    line_count,
                    numbers,
                    input,
                ),
                b'*' => operate(
                    1,
                    |a, b| a * b,
                    line_length,
                    col,
                    line_count,
                    numbers,
                    input,
                ),
                _ => panic!("invalid symbol {:?}", operator as char),
            };
            // dbg!(total);
            sum += total;
            col += 1 + numbers;
        }

        sum
    }
}

fn operate(
    mut total: u64,
    mut f: impl FnMut(u64, u64) -> u64,
    line_length: usize,
    col: usize,
    line_count: usize,
    numbers: usize,
    input: &[u8],
) -> u64 {
    for c in 0..numbers {
        let mut number = 0;
        for r in 0..line_count - 1 {
            let digit = input[r * line_length + col + c];
            let digit = match digit {
                b'0'..=b'9' => digit - b'0',
                b' ' => continue,
                _ => panic!("invalid digit"),
            };
            number *= 10;
            number += digit as u64;
        }
        total = f(total, number);
    }
    total
}
