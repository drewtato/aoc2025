use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut presses = 0;
        let mut v1 = Vec::new();
        let mut v2 = Vec::new();
        lines(input, |machine| {
            presses += machine.enable_machine(&mut v1, &mut v2)
        });
        presses
    }

    fn part_two(_input: &[u8], _debug: u8) -> impl Display + 'static {
        "unimplemented"
    }
}

fn lines(input: &[u8], mut f: impl FnMut(Machine)) {
    let mut input = Consume::new(input);
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
        f(Machine {
            lights,
            buttons,
            joltages,
        });
    }
}

const MAX_BUTTONS: usize = 14;

#[allow(dead_code)]
#[derive(Clone)]
struct Machine {
    lights: StateInt,
    buttons: ArrayVec<StateInt, MAX_BUTTONS>,
    joltages: ArrayVec<u16, 10>,
}

type StateInt = u16;

struct State {
    lights: StateInt,
    used: u8,
}
impl State {
    fn new(lights: StateInt, used: u8) -> Self {
        Self { lights, used }
    }
}

impl Machine {
    fn enable_machine<'a>(
        self,
        mut states: &'a mut Vec<State>,
        mut states_tmp: &'a mut Vec<State>,
    ) -> usize {
        states.clear();
        states_tmp.clear();
        states.push(State::new(self.lights, 0));
        // eprintln!("{self:?}");
        // return 0;
        for presses in 1.. {
            for &State { lights, used } in &*states {
                for (i, &button) in self.buttons.iter().enumerate() {
                    if i as u8 == used {
                        continue;
                    }
                    let new_lights = press_button(lights, button);
                    // eprintln!(
                    //     "apply {:?} to {:?} = {:?}",
                    //     StatePrinter(button),
                    //     StatePrinter(lights),
                    //     StatePrinter(new_lights),
                    // );
                    if new_lights == 0 {
                        // eprintln!("  {presses}");
                        return presses;
                    }
                    states_tmp.push(State::new(new_lights, i as u8));
                }
            }
            states.clear();
            swap(&mut states, &mut states_tmp);

            if states.is_empty() {
                panic!("no states left");
            }
            // eprintln!(
            //     "{:?}",
            //     states.iter().map(|&(s, _)| StatePrinter(s)).collect_vec()
            // );
        }
        unreachable!();
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

fn press_button(state: StateInt, button: StateInt) -> StateInt {
    state ^ button
}
