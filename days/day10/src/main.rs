use helpers::*;
// use rayon::prelude::*;

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
        macro_rules! specific {
            ($($n:literal),* $(,)?) => {
                match self.joltages.len() {
                    $(
                        $n => SpecificLengthMachine::<$n>::try_from(self)
                            .unwrap()
                            .configure_joltage(),
                    )*
                    n => panic!("please add length {n} to the match"),
                }
            };
        }
        specific!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    }
}

#[derive(Debug, Clone)]
struct SpecificLengthMachine<const N: usize> {
    buttons: ArrayVec<ButtonInt, MAX_BUTTONS>,
    joltages: [u16; N],
}

impl<const N: usize> TryFrom<Machine> for SpecificLengthMachine<N> {
    type Error = &'static str;

    fn try_from(machine: Machine) -> Result<Self, Self::Error> {
        Ok(Self {
            buttons: machine.buttons,
            joltages: *machine
                .joltages
                .as_array()
                .ok_or("wrong number of joltages")?,
        })
    }
}

#[derive(Debug)]
struct State<const N: usize> {
    heuristic_and_cost: u16,
    cost: u16,
    node: [u16; N],
    last_button: u16,
}

impl<const N: usize> PartialEq for State<N> {
    fn eq(&self, other: &Self) -> bool {
        self.heuristic_and_cost == other.heuristic_and_cost
    }
}

impl<const N: usize> Eq for State<N> {}

impl<const N: usize> PartialOrd for State<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for State<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.heuristic_and_cost.cmp(&other.heuristic_and_cost)
    }
}

impl<const N: usize> SpecificLengthMachine<N> {
    fn configure_joltage(self) -> u32 {
        let mut queue = BinaryHeap::new_min();
        let start = [0; N];
        let cost = 0;
        let heuristic = self.max_joltage_diff(start).unwrap();
        queue.push(State {
            heuristic_and_cost: heuristic + cost,
            cost,
            node: start,
            last_button: 0,
        });
        let mut seen = HashSet::default();
        seen.insert(start);

        while let Some(State {
            heuristic_and_cost,
            cost,
            node,
            last_button,
        }) = queue.pop()
        {
            seen.insert(node);
            let new_cost = cost + 1;
            let next_button = last_button as usize + 1;
            if next_button < self.buttons.len() {
                queue.push(State {
                    heuristic_and_cost,
                    cost,
                    node,
                    last_button: next_button as _,
                });
            }
            let button = self.buttons[last_button as usize];
            let new_node = jolt(node, button);
            let Some(heuristic) = self.max_joltage_diff(new_node) else {
                continue;
            };
            if heuristic == 0 {
                return new_cost as _;
            }
            if seen.contains(&new_node) {
                continue;
            }
            queue.push(State {
                heuristic_and_cost: heuristic + cost,
                cost: new_cost,
                node: new_node,
                last_button,
            });
        }

        panic!("no path");
    }

    fn max_joltage_diff(&self, node: [u16; N]) -> Option<u16> {
        let mut max = 0;
        for (goal, n) in self.joltages.into_iter().zip(node) {
            let diff = goal.checked_sub(n)?;
            max = max.max(diff);
        }
        Some(max)
    }
}

fn jolt<const N: usize>(mut node: [u16; N], mut button: u16) -> [u16; N] {
    for n in node.iter_mut().rev() {
        if button.is_odd() {
            *n += 1;
        }
        button /= 2;
    }
    node
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
