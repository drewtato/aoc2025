use std::sync::atomic::AtomicUsize;

use helpers::*;
use rayon::prelude::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        lines(input, |machine| machine.enable_machine())
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        lines(input, |machine| machine.configure_joltage())
    }
}

fn lines(input: &[u8], f: impl Fn(Machine) -> u32 + Send + Sync) -> u32 {
    let mut input = Consume::new(input);
    let mut machines = Vec::new();
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
        machines.push(Machine {
            lights,
            buttons,
            joltages,
        });
    }
    let progress = AtomicUsize::new(0);
    let total = machines.len();
    machines
        .into_par_iter()
        .enumerate()
        .map(|(i, machine)| {
            let buttons = machine.buttons.len();
            let presses = f(machine);
            let current = progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            eprintln!("done with {i} ({buttons} buttons) ({current}/{total})");
            presses
        })
        .sum()
}

const MAX_BUTTONS: usize = 14;
const MAX_JOLTAGES: usize = 10;

#[allow(dead_code)]
#[derive(Clone)]
struct Machine {
    lights: StateInt,
    buttons: ArrayVec<StateInt, MAX_BUTTONS>,
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
        min_presses(
            &mut self
                .joltages
                .iter()
                .map(|&j| j as i16)
                .collect::<ArrayVec<_, MAX_JOLTAGES>>(),
            &self.buttons,
        )
        .unwrap()
    }
}

fn min_presses(joltages: &mut [i16], buttons: &[u16]) -> Option<u32> {
    let Some((&button, rest_buttons)) = buttons.split_first() else {
        if joltages.iter().all(|&j| j == 0) {
            return Some(0);
        } else {
            return None;
        }
    };
    let mut best_presses = None;
    for presses in 0.. {
        if joltages.iter().any(|j| j.is_negative()) {
            unjolt(joltages, button, presses);
            break;
        }
        if let Some(later) = min_presses(joltages, rest_buttons) {
            best_presses = Some(
                best_presses
                    .unwrap_or_else(|| presses + later)
                    .min(presses + later),
            );
        };
        jolt(joltages, button);
    }
    best_presses
}

fn jolt(joltages: &mut [i16], mut button: u16) {
    for j in joltages.iter_mut().rev() {
        if button.is_odd() {
            *j -= 1;
        }
        button /= 2;
    }
}

fn unjolt(joltages: &mut [i16], mut button: u16, presses: u32) {
    for j in joltages.iter_mut().rev() {
        if button.is_odd() {
            *j += presses as i16;
        }
        button /= 2;
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
