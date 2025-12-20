use std::borrow::Borrow;

use helpers::petgraph::algo::toposort;
use helpers::petgraph::graphmap::DiGraphMap;
use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let graph = make_graph(input);
        let sorted = toposort(&graph, None).unwrap();
        let mut counts = HashMap::with_capacity_and_hasher(sorted.len(), Default::default());
        for node in sorted {
            if node == b"you" {
                counts.insert(node, 1);
            }
            let count: u64 = *counts.entry(node).or_default();
            for neighbor in graph.neighbors(node) {
                *counts.entry(neighbor).or_default() += count;
            }
        }
        counts[b"out"]
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let graph = make_graph(input);
        let sorted = toposort(&graph, None).unwrap();
        let (first, second): (Label, Label) = {
            if sorted.iter().position(|node| node == b"fft")
                < sorted.iter().position(|node| node == b"dac")
            {
                (b"fft".into(), b"dac".into())
            } else {
                (b"dac".into(), b"fft".into())
            }
        };
        let mut counts = HashMap::with_capacity_and_hasher(sorted.len(), Default::default());
        for node in sorted {
            // [seen none, seen fft, seen fft+dac]
            let [count, fft, dac] = counts.entry(node).or_default();
            if node == b"svr" {
                *count = 1u64;
            } else if node == first {
                *fft = *count;
            } else if node == second {
                *dac = *fft;
            }
            let [count, fft, dac] = [*count, *fft, *dac];
            for neighbor in graph.neighbors(node) {
                let [ncount, nfft, ndac] = counts.entry(neighbor).or_default();
                *ncount += count;
                *nfft += fft;
                *ndac += dac
            }
        }
        counts[b"out"][2]
    }
}

fn make_graph(input: &[u8]) -> DiGraphMap<Label, ()> {
    let mut graph = DiGraphMap::new();
    let mut input = Consume::new(input);
    while !input.is_empty() {
        let node = Label::new(input.consume_array().unwrap());
        assert!(input.byte(b':'));
        while input.byte(b' ') {
            let destination = Label::new(input.consume_array().unwrap());
            assert!(graph.add_edge(node, destination, ()).is_none());
        }
        assert!(input.byte(b'\n'));
    }

    graph
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Label([u8; 3]);

impl From<&[u8; 3]> for Label {
    fn from(value: &[u8; 3]) -> Self {
        Self::new(*value)
    }
}

impl From<[u8; 3]> for Label {
    fn from(value: [u8; 3]) -> Self {
        Self::new(value)
    }
}

impl TryFrom<&[u8]> for Label {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self::new(value.try_into().map_err(|_| "wrong length")?))
    }
}

impl Borrow<[u8]> for Label {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8; 3]> for Label {
    fn borrow(&self) -> &[u8; 3] {
        &self.0
    }
}

impl PartialEq<[u8]> for Label {
    fn eq(&self, other: &[u8]) -> bool {
        if other.len() == 3 {
            *self == Label::new(other.try_into().unwrap())
        } else {
            false
        }
    }
}

impl PartialEq<&[u8; 3]> for Label {
    fn eq(&self, other: &&[u8; 3]) -> bool {
        *self == Label::new(**other)
    }
}

impl PartialEq<[u8; 3]> for Label {
    fn eq(&self, other: &[u8; 3]) -> bool {
        *self == Label::new(*other)
    }
}

impl Label {
    fn new(label: [u8; 3]) -> Self {
        Self(label)
    }
}

impl Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.0[0] as char, self.0[1] as char, self.0[2] as char
        )
    }
}
