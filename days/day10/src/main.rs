use helpers::*;
use z3::Optimize;
use z3::ast::Int;
// use rayon::prelude::*;

fn main() {
    use solver_interface::ChildSolverExt;
    SolverAoc::run().unwrap_display();
}

struct SolverAoc;

impl solver_interface::ChildSolver for SolverAoc {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        lines(input, |machine| machine.enable_machine())
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        lines(input, |machine| machine.configure_joltage())
    }
}

fn lines(input: &[u8], f: impl Fn(Machine) -> u32 + Send + Sync) -> u32 {
    let mut input = Consume::new(input);
    // let mut machines = Vec::new();
    let mut sum = 0;
    while !input.is_empty() {
        assert!(input.byte(b'['));
        let mut lights = 0;
        let mut len = 0;
        loop {
            let b = input.consume_byte().unwrap();
            match b {
                b'.' => lights <<= 1,
                b'#' => {
                    lights <<= 1;
                    lights += 1;
                }
                b']' => break,
                _ => panic!("invalid light: {}", b as char),
            }
            len += 1;
        }

        let mut buttons = ArrayVec::new();
        loop {
            assert!(input.byte(b' '));
            if input.consume_byte().unwrap() == b'{' {
                break;
            }

            let mut button = 0;
            loop {
                button |= 1 << (len - 1 - input.int::<u64>().unwrap());
                if input.consume_byte().unwrap() == b')' {
                    break;
                }
            }
            buttons.push(button);
        }

        let mut joltages = ArrayVec::new();
        loop {
            joltages.push(input.int().unwrap());
            if input.consume_byte().unwrap() == b'}' {
                break;
            }
        }

        assert!(input.newline());
        let machine = Machine {
            lights,
            buttons,
            joltages,
        };
        // machines.push(machine);
        sum += f(machine);
    }

    // machines.into_par_iter().map(f).sum()
    sum
}

const MAX_BUTTONS: usize = 14;
const MAX_JOLTAGES: usize = 10;

#[derive(Clone)]
struct Machine {
    lights: StateInt,
    buttons: ArrayVec<ButtonInt, MAX_BUTTONS>,
    joltages: ArrayVec<u16, MAX_JOLTAGES>,
}

type StateInt = u16;
type ButtonInt = u16;

impl Machine {
    fn enable_machine(self) -> u32 {
        let mut min_enabled = u32::MAX;
        let end: ButtonInt = (1 << self.buttons.len()) - 1;
        for button_mask in 0..end {
            let lights = self.push_buttons(button_mask);
            if self.lights == lights {
                min_enabled = min_enabled.min(button_mask.count_ones());
            }
        }

        min_enabled
    }

    fn push_buttons(&self, mut button_mask: u16) -> u16 {
        let mut lights = 0;
        for &button in &self.buttons {
            if button_mask.is_odd() {
                lights ^= button;
            }
            button_mask /= 2;
        }
        lights
    }

    fn configure_joltage(self) -> u32 {
        let solver = Optimize::new();
        let buttons_decomposed: ArrayVec<_, MAX_BUTTONS> = self
            .buttons
            .into_iter()
            .enumerate()
            .map(|(variable, mut b)| {
                let mut button: ArrayVec<bool, MAX_JOLTAGES> = fn_iter(|| {
                    let a = b.is_odd();
                    b /= 2;
                    Some(a)
                })
                .collect();
                button.reverse();
                (button, Int::fresh_const(&format!("button{variable}")))
            })
            .collect();

        let button_presses = buttons_decomposed
            .iter()
            .map(|(_, a)| a.clone())
            .reduce(|a, b| a + b)
            .unwrap();
        solver.minimize(&button_presses);
        for (_, b) in &buttons_decomposed {
            solver.assert(&b.ge(0));
        }

        for (i, j) in self.joltages.into_iter().enumerate() {
            let lhs = buttons_decomposed
                .iter()
                .fold(
                    Int::from_u64(0),
                    |int, (b_arr, b_int)| {
                        if b_arr[i] { int + b_int } else { int }
                    },
                );
            solver.assert(&lhs.eq(j));
        }

        solver.check(&[]);
        let model = solver.get_model().unwrap();
        let mut sum = 0;
        for (_, button) in buttons_decomposed {
            let value = model
                .eval(&button, false)
                .expect("model failed")
                .as_u64()
                .expect("wasn't a u64");
            // eprintln!("  {value}");
            sum += value as u32;
        }

        // eprintln!("{sum}");

        sum
    }
}

impl Debug for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Machine")
            .field("lights", &StatePrinter(self.lights))
            .field(
                "buttons",
                &self
                    .buttons
                    .iter()
                    .map(|&b| StatePrinter(b))
                    .collect::<ArrayVec<_, MAX_BUTTONS>>(),
            )
            .field("joltages", &self.joltages)
            .finish()
    }
}

struct StatePrinter(StateInt);

impl Debug for StatePrinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:b}", self.0)
    }
}
